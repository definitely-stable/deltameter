// External comparator harness. Built INSIDE the exact pinned official riblt
// checkout, using its own go.mod/go.sum. Never a DeltaMeter runtime dependency.
package main

import (
    "bufio"
    "encoding/binary"
    "encoding/hex"
    "fmt"
    "os"
    "sort"
    "strconv"
    "strings"

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
func require(ok bool) { if !ok { panic("invalid harness input/state") } }
func keys(text string) []item {
    result := []item{}
    if text == "-" { return result }
    for _, s := range strings.Split(text, ",") {
        x, err := strconv.ParseUint(s, 16, 64)
        require(err == nil && (len(result) == 0 || uint64(result[len(result)-1]) < x))
        result = append(result, item(x))
    }
    require(len(result) <= 1048576)
    return result
}
func keyText(values []riblt.HashedSymbol[item]) string {
    xs := make([]uint64, len(values))
    for i, v := range values { xs[i] = uint64(v.Symbol) }
    sort.Slice(xs, func(i, j int) bool { return xs[i] < xs[j] })
    parts := []string{}
    for i, v := range xs {
        require(i == 0 || xs[i-1] < v)
        parts = append(parts, strconv.FormatUint(v, 16))
    }
    if len(parts) == 0 { return "-" }
    return strings.Join(parts, ",")
}
func main() {
    scan := bufio.NewScanner(os.Stdin)
    scan.Buffer(make([]byte, 4096), 40000000)
    out := bufio.NewWriter(os.Stdout)
    var enc riblt.Encoder[item]
    var dec riblt.Decoder[item]
    produced, consumed := 0, 0
    initialized := false
    for scan.Scan() {
        p := strings.Fields(scan.Text())
        require(len(p) >= 2)
        switch p[0] {
        case "init":
            require(len(p) == 3)
            // New scheduler per session; no unsupported post-stream mutation.
            enc = riblt.Encoder[item]{}
            dec = riblt.Decoder[item]{}
            for _, k := range keys(p[1]) { enc.AddSymbol(k) }
            for _, k := range keys(p[2]) { dec.AddSymbol(k) }
            produced, consumed, initialized = 0, 0, true
            fmt.Fprintln(out, "ready")
        case "next":
            require(initialized && len(p) == 2 && produced == consumed)
            target, err := strconv.Atoi(p[1])
            require(err == nil && target > produced && target <= 1024)
            buf := make([]byte, 24*(target-produced))
            for off := 0; off < len(buf); off += 24 {
                c := enc.ProduceNextCodedSymbol()
                binary.LittleEndian.PutUint64(buf[off:], uint64(c.Symbol))
                binary.LittleEndian.PutUint64(buf[off+8:], c.Hash)
                binary.LittleEndian.PutUint64(buf[off+16:], uint64(c.Count))
            }
            produced = target
            fmt.Fprintln(out, hex.EncodeToString(buf))
        case "consume":
            require(initialized && len(p) == 2)
            buf, err := hex.DecodeString(p[1])
            require(err == nil && len(buf) == 24*(produced-consumed) && len(buf) > 0)
            for off := 0; off < len(buf); off += 24 {
                c := riblt.CodedSymbol[item]{HashedSymbol: riblt.HashedSymbol[item]{
                    Symbol: item(binary.LittleEndian.Uint64(buf[off:])),
                    Hash: binary.LittleEndian.Uint64(buf[off+8:]),
                }, Count: int64(binary.LittleEndian.Uint64(buf[off+16:]))}
                dec.AddCodedSymbol(c)
                dec.TryDecode()
            }
            consumed = produced
            if dec.Decoded() {
                fmt.Fprintln(out, "ok " + keyText(dec.Remote()) + " " + keyText(dec.Local()))
            } else { fmt.Fprintln(out, "reject") }
        default:
            panic("invalid harness command")
        }
        require(out.Flush() == nil)
    }
    require(scan.Err() == nil)
}
