pub export const AllocInfo = extern struct {
    address: i32,
    size: i32,
};

pub export fn malloc(size: c_int) callconv(.C) anyopaque {
    _ = size;
}

pub export fn free(pointer: anyopaque) callconv(.C) anyopaque {
    _ = pointer;
}

pub export fn realloc(pointer: anyopaque, size: c_int) callconv(.C) anyopaque {
    _ = size;
    _ = pointer;
}

pub export fn calloc(source: c_int, destination: c_int) callconv(.C) anyopaque {
    _ = destination;
    _ = source;
}
