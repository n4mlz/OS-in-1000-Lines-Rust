use core::arch::asm;

#[allow(clippy::too_many_arguments)]
pub fn sbi_call(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    fid: usize,
    eid: usize,
) -> Result<usize, isize> {
    let mut err: isize;
    let mut value: usize;

    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0 => err,
            inout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid,
        );
    }

    if err < 0 { Err(err) } else { Ok(value) }
}
