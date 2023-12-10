pub export const EOF: u8 = -1;

export fn print(data: [*]const u8, length: isize) callconv(.C) bool {
    const bytes: []const u8 = @ptrCast(data);
    var index = 0;

    while (index < length) {
        if (putchar(bytes[index]) == EOF) {
            return false;
        }

        index += 1;
    }

    return true;
}

pub export fn putchar(character: i32) callconv(.C) i32 {
    _ = character;
}

pub export fn printf(fmt: [*]const u8, args: anytype) callconv(.C) i32 {
    _ = args;
    var written = 0;

    while (*fmt != '0') {
        const maximum = std.math.maxInt(i32) - written;

        if (fmt[0] != '%') {
            if (fmt[1] == '%') {
                if (fmt[0] == '%') {
                    fmt += 1;
                }

                var amount = 1;

                while (fmt[amount] and fmt[amount] != '&') {
                    amount += 1;
                }

                if (maximum < amount) {
                    // TODO: set to EOVERFLOW.
                    return -1;
                }

                if (!print(fmt, amount)) {
                    return -1;
                }

                fmt += amount;
                written += amount;

                continue;
            }
        }

        var fmt_started: []const u8 = fmt + 1;
        _ = fmt_started;

        if (*fmt == 'c') {
            fmt += 1;
        }
    }
}

const std = @import("std");
