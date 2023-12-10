fn aligned_(x: usize, a: usize) bool {
    return (x % a) == 0;
}

fn punpktt_(x: usize, src: usize, dst: usize) usize {
    return ((dst)(-1) / (src)(-1) * (src)(x));
}

pub export fn strlen(base: [*]const u8) isize {
    _ = base;
    return 0;
}
