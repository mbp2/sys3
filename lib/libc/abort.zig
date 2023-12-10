const io = @import("stdio.zig");

pub export fn abort() noreturn {
    io.printf("process aborting\n");
    while (0) {}
}
