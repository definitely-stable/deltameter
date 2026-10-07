// M6-D13-B0 external Rateless IBLT readiness adapter.
//
// This file is copied into the exact pinned upstream riblt checkout and built
// against its locked go.mod/go.sum. It is never a DeltaMeter runtime dependency.
package main

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/dchest/siphash"
	"github.com/yangl1996/riblt"
)

type item uint64

func (x item) XOR(y item) item { return x ^ y }

func (x item) Hash() uint64 {
	var buf [8]byte
	binary.LittleEndian.PutUint64(buf[:], uint64(x))
	return siphash.Hash(123, 456, buf[:])
}

type state struct {
	a      []item
	b      []item
	clkTck uint64
}

func require(ok bool) {
	if !ok {
		panic("invalid harness input/state")
	}
}

func parseKeys(text string) []item {
	if text == "-" {
		return []item{}
	}
	parts := strings.Split(text, ",")
	out := make([]item, len(parts))
	for i, part := range parts {
		value, err := strconv.ParseUint(part, 16, 64)
		require(err == nil)
		out[i] = item(value)
		if i != 0 {
			require(out[i-1] < out[i])
		}
	}
	require(len(out) <= 1048576)
	return out
}

func copyKeys(values []item) []item {
	out := make([]item, len(values))
	copy(out, values)
	return out
}

func equalKeys(left, right []item) bool {
	if len(left) != len(right) {
		return false
	}
	for i := range left {
		if left[i] != right[i] {
			return false
		}
	}
	return true
}

func update(values []item, operation string, key item) ([]item, bool) {
	index := sort.Search(len(values), func(i int) bool { return values[i] >= key })
	switch operation {
	case "insert":
		if index < len(values) && values[index] == key {
			return values, false
		}
		values = append(values, 0)
		copy(values[index+1:], values[index:])
		values[index] = key
		return values, true
	case "delete":
		if index == len(values) || values[index] != key {
			return values, false
		}
		copy(values[index:], values[index+1:])
		values = values[:len(values)-1]
		return values, true
	default:
		panic("invalid operation")
	}
}

func applyCandidate(base []item, remote, local []riblt.HashedSymbol[item]) []item {
	out := copyKeys(base)
	for _, value := range local {
		var ok bool
		out, ok = update(out, "delete", value.Symbol)
		require(ok)
	}
	for _, value := range remote {
		var ok bool
		out, ok = update(out, "insert", value.Symbol)
		require(ok)
	}
	return out
}

func encodeList(values []item) []byte {
	out := make([]byte, 8+8*len(values))
	binary.LittleEndian.PutUint64(out[:8], uint64(len(values)))
	for i, value := range values {
		binary.LittleEndian.PutUint64(out[8+i*8:], uint64(value))
	}
	return out
}

func processCPUTicks() uint64 {
	raw, err := os.ReadFile("/proc/self/stat")
	require(err == nil)
	text := string(raw)
	close := strings.LastIndex(text, ")")
	require(close >= 0)
	fields := strings.Fields(text[close+2:])
	require(len(fields) > 12)
	user, err := strconv.ParseUint(fields[11], 10, 64)
	require(err == nil)
	system, err := strconv.ParseUint(fields[12], 10, 64)
	require(err == nil)
	return user + system
}

func clockTicks() uint64 {
	raw, err := exec.Command("getconf", "CLK_TCK").Output()
	require(err == nil)
	value, err := strconv.ParseUint(strings.TrimSpace(string(raw)), 10, 64)
	require(err == nil && value != 0)
	return value
}

func rssBytes() (uint64, uint64) {
	raw, err := os.ReadFile("/proc/self/status")
	require(err == nil)
	var rss, hwm uint64
	for _, line := range strings.Split(string(raw), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 2 {
			continue
		}
		switch fields[0] {
		case "VmRSS:":
			value, err := strconv.ParseUint(fields[1], 10, 64)
			require(err == nil)
			rss = value * 1024
		case "VmHWM:":
			value, err := strconv.ParseUint(fields[1], 10, 64)
			require(err == nil)
			hwm = value * 1024
		}
	}
	require(rss != 0 && hwm >= rss)
	return rss, hwm
}

func kv(name string, value any) string {
	return fmt.Sprintf("%s=%v", name, value)
}

func syncSession(s *state, lane string, limit int) string {
	require(lane == "pull" || lane == "stream")
	require(limit >= 1 && limit <= 1024 && limit&(limit-1) == 0)

	cpu0 := processCPUTicks()

	encImportStart := time.Now()
	enc := riblt.Encoder[item]{}
	for _, value := range s.a {
		enc.AddSymbol(value)
	}
	encImportNS := time.Since(encImportStart).Nanoseconds()

	decImportStart := time.Now()
	dec := riblt.Decoder[item]{}
	for _, value := range s.b {
		dec.AddSymbol(value)
	}
	decImportNS := time.Since(decImportStart).Nanoseconds()

	produceNS := int64(0)
	decodeNS := int64(0)
	cells := 0
	batches := 0
	decoded := false

	if lane == "stream" {
		for cells < limit {
			started := time.Now()
			coded := enc.ProduceNextCodedSymbol()
			produceNS += time.Since(started).Nanoseconds()

			started = time.Now()
			dec.AddCodedSymbol(coded)
			dec.TryDecode()
			decodeNS += time.Since(started).Nanoseconds()

			cells++
			if dec.Decoded() {
				decoded = true
				break
			}
		}
		batches = 1
	} else {
		produced := 0
		target := 1
		buffer := make([]riblt.CodedSymbol[item], 0, limit)
		for target <= limit {
			for produced < target {
				started := time.Now()
				buffer = append(buffer, enc.ProduceNextCodedSymbol())
				produceNS += time.Since(started).Nanoseconds()
				produced++
			}

			started := time.Now()
			for cells < target {
				dec.AddCodedSymbol(buffer[cells])
				dec.TryDecode()
				cells++
			}
			decodeNS += time.Since(started).Nanoseconds()
			batches++

			if dec.Decoded() {
				decoded = true
				break
			}
			target *= 2
		}
	}

	remoteCap := 0
	localCap := 0
	applyNS := int64(0)
	verifyNS := int64(0)
	fallbackNS := int64(0)
	fallback := 0

	if decoded {
		remote := dec.Remote()
		local := dec.Local()
		remoteCap = cap(remote)
		localCap = cap(local)

		started := time.Now()
		s.b = applyCandidate(s.b, remote, local)
		applyNS = time.Since(started).Nanoseconds()

		started = time.Now()
		_ = encodeList(s.b)
		verified := equalKeys(s.a, s.b)
		verifyNS = time.Since(started).Nanoseconds()
		if !verified {
			fallback = 1
		}
	} else {
		fallback = 1
	}

	if fallback != 0 {
		started := time.Now()
		s.b = copyKeys(s.a)
		fallbackNS = time.Since(started).Nanoseconds()
	}

	require(equalKeys(s.a, s.b))

	nativeTotal := encImportNS + decImportNS + produceNS + decodeNS + applyNS + verifyNS + fallbackNS
	cpuTicks := processCPUTicks() - cpu0
	var mem runtime.MemStats
	runtime.ReadMemStats(&mem)
	rss, hwm := rssBytes()

	fields := []string{
		"sync",
		kv("lane", map[string]int{"pull": 0, "stream": 1}[lane]),
		kv("exact", 1),
		kv("fallback", fallback),
		kv("cells", cells),
		kv("batches", batches),
		kv("encoder_import_ns", encImportNS),
		kv("decoder_import_ns", decImportNS),
		kv("produce_ns", produceNS),
		kv("decode_ns", decodeNS),
		kv("apply_ns", applyNS),
		kv("verification_prepare_ns", verifyNS),
		kv("fallback_ns", fallbackNS),
		kv("native_total_ns", nativeTotal),
		kv("cpu_ticks", cpuTicks),
		kv("clk_tck", s.clkTck),
		kv("source_a_len", len(s.a)),
		kv("source_a_cap", cap(s.a)),
		kv("source_b_len", len(s.b)),
		kv("source_b_cap", cap(s.b)),
		kv("remote_cap", remoteCap),
		kv("local_cap", localCap),
		kv("runtime_alloc_bytes", mem.Alloc),
		kv("runtime_heap_sys_bytes", mem.HeapSys),
		kv("vmrss_bytes", rss),
		kv("vmhwm_bytes", hwm),
	}
	return strings.Join(fields, " ")
}


func initializeState(s *state, left, right []item) string {
	started := time.Now()
	s.a = copyKeys(left)
	s.b = copyKeys(right)
	buildNS := time.Since(started).Nanoseconds()
	var mem runtime.MemStats
	runtime.ReadMemStats(&mem)
	rss, hwm := rssBytes()
	return strings.Join([]string{
		"ready",
		kv("source_build_ns", buildNS),
		kv("clk_tck", s.clkTck),
		kv("source_len", len(s.a)),
		kv("source_a_cap", cap(s.a)),
		kv("source_b_cap", cap(s.b)),
		kv("runtime_alloc_bytes", mem.Alloc),
		kv("runtime_heap_sys_bytes", mem.HeapSys),
		kv("vmrss_bytes", rss),
		kv("vmhwm_bytes", hwm),
	}, " ")
}

func main() {
	scanner := bufio.NewScanner(os.Stdin)
	scanner.Buffer(make([]byte, 4096), 40000000)
	out := bufio.NewWriter(os.Stdout)
	s := state{clkTck: clockTicks()}

	for scanner.Scan() {
		parts := strings.Fields(scanner.Text())
		require(len(parts) >= 1)

		var response string
		switch parts[0] {
		case "init":
			require(len(parts) == 2)
			keys := parseKeys(parts[1])
			response = initializeState(&s, keys, keys)
		case "init_pair":
			require(len(parts) == 3)
			left := parseKeys(parts[1])
			right := parseKeys(parts[2])
			response = initializeState(&s, left, right)
		case "update":
			require(len(parts) == 3)
			value, err := strconv.ParseUint(parts[2], 16, 64)
			require(err == nil)
			before := copyKeys(s.a)
			cpu0 := processCPUTicks()
			started := time.Now()
			var ok bool
			s.a, ok = update(s.a, parts[1], item(value))
			elapsed := time.Since(started).Nanoseconds()
			ticks := processCPUTicks() - cpu0
			if !ok {
				require(equalKeys(before, s.a))
			}
			response = strings.Join([]string{
				"update",
				kv("ok", map[bool]int{false: 0, true: 1}[ok]),
				kv("exact_ns", elapsed),
				kv("native_total_ns", elapsed),
				kv("cpu_ticks", ticks),
				kv("source_len", len(s.a)),
				kv("source_cap", cap(s.a)),
			}, " ")
		case "sync":
			require(len(parts) == 3)
			limit, err := strconv.Atoi(parts[2])
			require(err == nil)
			response = syncSession(&s, parts[1], limit)
		case "check":
			require(len(parts) == 1)
			response = strings.Join([]string{
				"check",
				kv("equal", map[bool]int{false: 0, true: 1}[equalKeys(s.a, s.b)]),
			}, " ")
		case "quit":
			return
		default:
			panic("invalid command")
		}

		fmt.Fprintln(out, response)
		require(out.Flush() == nil)
	}
	require(scanner.Err() == nil)
}
