// G1-B2-B0 physical retained source comparison. No TCP, no public API.
// All network bytes/RTT are transparent analytical models, not measured links.
// Public MASTER_KEY is a fixture; correctness is not keyed-PRF security.
const B0_NS: [usize; 3] = [256, 4096, 65_536];
const B0_DS: [usize; 6] = [0, 16, 48, 57, 64, 65];
const B0_SESSIONS: [usize; 3] = [1, 10, 100];
const B0_REPEATS: usize = 3;
const B0_T: u64 = 64;
const B0_HEADER: usize = 64;
const B0_REQUEST: usize = 24;
const B0_UNIT_CONTEXT: &str = "deltameter 2026-10-09 deltaguard-g1b unit GF2 research v1";

#[derive(Clone, Copy)]
enum B0Kind {
    Near11,
    Near12,
    Unit1,
    J52,
}
impl B0Kind {
    const fn label(self) -> &'static str {
        match self {
            Self::Near11 => "b11",
            Self::Near12 => "b12",
            Self::Unit1 => "unit1",
            Self::J52 => "j52",
        }
    }
    const fn all() -> [Self; 4] {
        [Self::Near11, Self::Near12, Self::Unit1, Self::J52]
    }
}

struct B0UnitSide {
    key: [u8; 32],
    words: Box<[u64]>,
}
impl B0UnitSide {
    fn new() -> Self {
        Self {
            key: derive_key(B0_UNIT_CONTEXT, &MASTER_KEY),
            words: vec![0_u64; WORDS_PER_LEVEL].into_boxed_slice(),
        }
    }
    fn toggle(&mut self, token: u64) {
        let bytes = blake3::keyed_hash(&self.key, &token.to_le_bytes());
        let b = bytes.as_bytes();
        let level = u64::from_le_bytes(b[8..16].try_into().unwrap());
        if level == 0 || level.trailing_zeros() != 0 {
            return;
        }
        let row = (u64::from_le_bytes(b[..8].try_into().unwrap()) as usize) & (ROWS - 1);
        self.words[row / 64] ^= 1_u64 << (row % 64);
    }
}
enum B0State {
    Near(NearFullGuard, NearFullGuard),
    Unit(B0UnitSide, B0UnitSide),
    Full(PackedSketch, PackedSketch),
}
impl B0State {
    fn new(kind: B0Kind) -> Self {
        match kind {
            B0Kind::Near11 => Self::Near(
                NearFullGuard::new(11, B0_T, &MASTER_KEY),
                NearFullGuard::new(11, B0_T, &MASTER_KEY),
            ),
            B0Kind::Near12 => Self::Near(
                NearFullGuard::new(12, B0_T, &MASTER_KEY),
                NearFullGuard::new(12, B0_T, &MASTER_KEY),
            ),
            B0Kind::Unit1 => Self::Unit(B0UnitSide::new(), B0UnitSide::new()),
            B0Kind::J52 => Self::Full(
                PackedSketch::new(J, Layout::LevelMajor),
                PackedSketch::new(J, Layout::LevelMajor),
            ),
        }
    }
    fn toggle(&mut self, right: bool, token: u64) {
        match self {
            Self::Near(a, b) => if right { b.toggle(token) } else { a.toggle(token) },
            Self::Unit(a, b) => if right { b.toggle(token) } else { a.toggle(token) },
            Self::Full(a, b) => if right { b.toggle(token) } else { a.toggle(token) },
        }
    }
    fn owner_bytes(&self) -> usize {
        match self {
            Self::Near(a, _) => a.payload_bytes(),
            Self::Unit(a, _) => a.words.len() * 8,
            Self::Full(a, _) => a.state_bytes(),
        }
    }
    fn object_bytes(&self) -> usize {
        match self {
            Self::Near(_, _) => std::mem::size_of::<NearFullGuard>(),
            Self::Unit(_, _) => std::mem::size_of::<B0UnitSide>(),
            Self::Full(_, _) => std::mem::size_of::<PackedSketch>(),
        }
    }
    fn xor_words(&self) -> Vec<u64> {
        let (a, b): (&[u64], &[u64]) = match self {
            Self::Near(a,b) => (&a.words,&b.words),
            Self::Unit(a,b) => (&a.words,&b.words),
            Self::Full(a,b) => (&a.words,&b.words),
        };
        assert_eq!(a.len(), b.len());
        a.iter().zip(b).map(|(x,y)| x ^ y).collect()
    }
    fn reference(&self, delta: &[u64]) -> Vec<u64> {
        let mut single = match self {
            Self::Near(a, _) => Self::Near(
                NearFullGuard::new(a.b, B0_T, &MASTER_KEY),
                NearFullGuard::new(a.b, B0_T, &MASTER_KEY),
            ),
            Self::Unit(_, _) => Self::Unit(B0UnitSide::new(), B0UnitSide::new()),
            Self::Full(_, _) => Self::Full(
                PackedSketch::new(J, Layout::LevelMajor),
                PackedSketch::new(J, Layout::LevelMajor),
            ),
        };
        for &token in delta {
            single.toggle(false, token);
        }
        let v=single.xor_words();
        assert_eq!(v.len()*8,self.owner_bytes());
        v
    }
    fn count(&self, merged: &[u64]) -> u32 {
        match self {
            Self::Near(_,_) | Self::Unit(_,_) => merged.iter().map(|w| w.count_ones()).sum(),
            Self::Full(_,_) => 0, // J52 Q32 lookup deliberately not available in this B0 lane
        }
    }
    fn cutoff(kind: B0Kind, b2acuts: &B2ACutoffs) -> i32 {
        match kind {
            B0Kind::Near11 => b2acuts[&(64,11)],
            B0Kind::Near12 => b2acuts[&(64,12)],
            B0Kind::Unit1 => 13, // pinned D54 exact G1B1 unit, T64, level1
            B0Kind::J52 => -2, // NO Q32 classification invented
        }
    }
}

fn b0_diff(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result=Vec::new();
    let (mut i,mut j)=(0,0);
    while i<a.len()||j<b.len(){
        if j==b.len() || (i<a.len()&&a[i]<b[j]){
            result.push(a[i]);i+=1;
        }else if i==a.len()||b[j]<a[i]{
            result.push(b[j]);j+=1;
        }else{i+=1;j+=1;}
    }
    result
}
fn b0_inputs(worker:usize,ni:usize,di:usize,repeat:usize)->(Vec<u64>,Vec<u64>){
    let n=B0_NS[ni];
    let d=B0_DS[di];
    let base=(worker as u64 * 10_000_000_000)
        +(ni as u64 * 1_000_000_000)
        +(di as u64 * 100_000_000)
        +(repeat as u64 * 1_000_000);
    let a:Vec<u64>=(0..n as u64).map(|i|base+i).collect();
    let mut b:Vec<u64>=(0..(n-d/2) as u64).map(|i|base+i).collect();
    b.extend((0..d.div_ceil(2) as u64).map(|i|base+n as u64+i));
    assert_eq!(b0_diff(&a,&b).len(),d);
    (a,b)
}
fn b0_generation_token(worker:usize,ni:usize,di:usize,repeat:usize,generation:usize)->u64{
    100_000_000_000
    +worker as u64*10_000_000_000
    +ni as u64*1_000_000_000
    +di as u64*100_000_000
    +repeat as u64*1_000_000
    +generation as u64
}
fn b0_validate() {
    let mut a=B0State::new(B0Kind::Near11);
    let wrong=B0State::new(B0Kind::Near12);
    if let B0State::Near(ref mut left, _) = a
        && let B0State::Near(ref right, _) = wrong {
        assert_eq!(left.xor_assign(right), Err("incompatible-b-t-or-key"));
    }
    let a=vec![3,5,7,10]; let b=vec![3,5,8,11];
    assert_eq!(b0_diff(&a,&b),vec![7,8,10,11]);
    assert_eq!(b0_diff(&[],&[]),Vec::<u64>::new());
}
fn b0_run(path:&Path,worker:usize){
    assert!((1..=5).contains(&worker));
    let cuts=b2a_load(path);
    assert_eq!(cuts[&(64,11)],48);
    assert_eq!(cuts[&(64,12)],52);
    b0_validate();
    let mut records=0;
    for (ni,&n) in B0_NS.iter().enumerate() {
        for (di,&d) in B0_DS.iter().enumerate() {
            for rep in 0..B0_REPEATS{
                let (mut a,mut b)=b0_inputs(worker,ni,di,rep);
                let mut states:Vec<(B0Kind,B0State,u128)>=Vec::new();
                for kind in B0Kind::all(){
                    let begin=Instant::now();
                    let mut state=B0State::new(kind);
                    for &token in &a{state.toggle(false,token);}
                    for &token in &b{state.toggle(true,token);}
                    let build_ns=begin.elapsed().as_nanos();
                    states.push((kind,state,build_ns));
                }
                let mut last=1_usize;
                for &session in &B0_SESSIONS{
                    let start=Instant::now();
                    for generation in last..session{
                        let token=b0_generation_token(worker,ni,di,rep,generation);
                        // canonical vectors grow in strictly ascending order
                        assert!(a.last().is_none_or(|x|*x<token));
                        assert!(b.last().is_none_or(|x|*x<token));
                        a.push(token);b.push(token);
                    }
                    let direct_build_ns=start.elapsed().as_nanos();
                    let start=Instant::now();
                    let diff=b0_diff(&a,&b);
                    let direct_compare_ns=start.elapsed().as_nanos();
                    assert_eq!(diff.len(),d);
                    let direct_bytes=2*B0_HEADER+8*(a.len()+b.len());
                    for (kind,state,build_ns) in &mut states{
                        let start=Instant::now();
                        for generation in last..session{
                            let token=b0_generation_token(worker,ni,di,rep,generation);
                            state.toggle(false,token);
                            state.toggle(true,token);
                        }
                        let incremental_ns=start.elapsed().as_nanos();
                        let start=Instant::now();
                        let merged=state.xor_words();
                        let odd=state.count(&merged);
                        let query_ns=start.elapsed().as_nanos();
                        assert_eq!(merged,state.reference(&diff),"retained two-owner XOR mismatch");
                        let cutoff=B0State::cutoff(*kind,&cuts);
                        let safe=cutoff>=0 && odd<=cutoff as u32;

                        if cutoff>=0&&d<=cutoff as usize{
                            assert!(safe,"guaranteed SAFE S<=d failed");
                        }
                        let wire=2*(B0_HEADER+state.owner_bytes());
                        let resolved=if safe{wire}else{wire+B0_REQUEST+direct_bytes};
                        assert!(wire<resolved||safe);
                        let answer=if cutoff== -2{"na"}else if safe{"safe"}else{"unknown"};
                        println!(
"B0_SAMPLE worker={worker} ni={ni} di={di} rep={rep} N={n} d={d} session={session} profile={} owner_bytes={} struct_bytes={} init_ns={build_ns} inc_ns={incremental_ns} query_ns={query_ns} direct_build_ns={direct_build_ns} direct_compare_ns={direct_compare_ns} odd={odd} cutoff={cutoff} answer={answer} safe={} direct_bytes={direct_bytes} guard_bytes={wire} resolved_bytes={resolved} sources_a={} sources_b={} model_header={B0_HEADER} model_request={B0_REQUEST}",
                            kind.label(),state.owner_bytes(),state.object_bytes(),
                            u8::from(safe),a.len(),b.len()
                        );
                        records+=1;
                    }
                    last=session;
                }
            }
        }
    }
    assert_eq!(records,648);
    println!("DELTAGUARD_B2B0_WORKER_PASS worker={worker} records={records}");
}
