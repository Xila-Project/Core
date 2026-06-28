use crate::host::wasi::context::WasiContext;

pub fn args_get(
    ctx: &mut WasiContext,
    memory: &mut [u8],
    argv: i32,     // pointer to argv array
    argv_buf: i32, // pointer to string buffer
) -> Result<i32, Errno> {
    let mut argv_offset = argv as usize;
    let mut buf_offset = argv_buf as usize;

    for arg in &ctx.args {
        // Write pointer to argv array
        let ptr = buf_offset as i32;
        memory[argv_offset..argv_offset + 4].copy_from_slice(&ptr.to_le_bytes());
        argv_offset += 4;

        // Write string to buffer
        memory[buf_offset..buf_offset + arg.len()].copy_from_slice(arg);
        memory[buf_offset + arg.len()] = 0; // null-terminate
        buf_offset += arg.len() + 1;
    }

    Ok(0) // success
}

pub fn args_sizes_get(
    ctx: &WasiContext,
    memory: &mut [u8],
    offset0: i32, // where to store count
    offset1: i32, // where to store buffer size
) -> Result<i32, Errno> {
    let argc = ctx.args.len() as i32;
    let argv_buf_size = ctx.args.iter().map(|a| a.len() + 1).sum::<usize>() as i32;

    memory[offset0 as usize..offset0 as usize + 4].copy_from_slice(&argc.to_le_bytes());
    memory[offset1 as usize..offset1 as usize + 4].copy_from_slice(&argv_buf_size.to_le_bytes());

    Ok(0)
}

pub fn environ_get(
    ctx: &mut WasiContext,
    memory: &mut [u8],
    environ: i32,
    environ_buf: i32,
) -> Result<i32, Errno> {
    let mut environ_offset = environ as usize;
    let mut buf_offset = environ_buf as usize;

    for (key, value) in &ctx.env {
        let ptr = buf_offset as i32;
        memory[environ_offset..environ_offset + 4].copy_from_slice(&ptr.to_le_bytes());
        environ_offset += 4;

        // Write "KEY=VALUE\0"
        memory[buf_offset..buf_offset + key.len()].copy_from_slice(key);
        buf_offset += key.len();
        memory[buf_offset] = b'=';
        buf_offset += 1;
        memory[buf_offset..buf_offset + value.len()].copy_from_slice(value);
        buf_offset += value.len();
        memory[buf_offset] = 0;
        buf_offset += 1;
    }

    Ok(0)
}
