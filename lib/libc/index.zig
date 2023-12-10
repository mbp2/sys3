const std = @import("std");
const testing = std.testing;

pub const abort = @import("abort.zig");
pub const alloc = @import("alloc.zig");
pub const io = @import("stdio.zig");
pub const memory = @import("memory.zig");
pub const string = @import("string.zig");

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try testing.expect(add(3, 7) == 10);
}
