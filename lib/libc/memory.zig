pub export fn memcmp(p1: anyopaque, p2: anyopaque, size: isize) callconv(.C) i32 {
    const a: *const u8 = @ptrCast(p1);
    const b: *const u8 = @ptrCast(p2);
    var index: isize = 0;

    while (index < size) {
        if (a[index] < b[index]) {
            return -1;
        } else if (b[index] < a[index]) {
            return 1;
        }

        index += 1;
    }

    return 0;
}

pub export fn memcpy(dst: anyopaque, src: anyopaque, size: isize) callconv(.C) anyopaque {
    var index: isize = 0;
    var dstPointer: *u8 = @ptrCast(dst);
    const srcPointer: *const u8 = @ptrCast(src);

    while (index < size) {
        dstPointer[index] = srcPointer[index];

        index += 1;
    }

    return dstPointer;
}

pub export fn memmove(dst: anyopaque, src: anyopaque, size: isize) callconv(.C) anyopaque {
    var dstPointer: *u8 = @ptrCast(dst);
    const srcPointer: *const u8 = @ptrCast(src);

    if (dstPointer < srcPointer) {
        var index: isize = 0;
        while (index < size) {
            dstPointer[index] = srcPointer[index];
            index += 1;
        }
    } else {
        var index: isize = size;
        while (index != 0) {
            dstPointer[index - 1] = srcPointer[index - 1];
            index -= 1;
        }
    }

    return dstPointer;
}

pub export fn memset(buf: anyopaque, value: i32, size: isize) callconv(.C) anyopaque {
    var bufPointer: *u8 = @ptrCast(buf);
    var index: isize = 0;

    while (index < size) {
        bufPointer[index] = @as(u8, value);
        index += 1;
    }

    return buf;
}
