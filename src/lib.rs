//! Safe Rust port of the LZO1X compression algorithm.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod compress;

use core::fmt::{self, Debug, Display};

/// Computes the worst case compressed size for the given input `size`.
pub fn worst_compress(size: usize) -> usize {
    size + (size / 16) + 64 + 3
}

/// Decompression error type.
#[derive(Debug)]
pub enum Error {
    /// Input does not have the correct format.
    Error,
    /// Expected more input.
    InputOverrun,
    /// Output was not large enough.
    OutputOverrun,
    /// Input bad format.
    LookbehindOverrun,
    /// Input bad format.
    InputNotConsumed,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Decompression result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Compress the given `input` to the `output` slice, returning a slice containing the compressed data.
pub fn compress_to_slice<'a>(input: &[u8], output: &'a mut [u8]) -> &'a mut [u8] {
    unsafe {
        let mut out_len: usize = 0;
        let mut wrkmem = [0u8; 8192 * 16];

        compress::lzo1x_1_compress(
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            (&mut out_len) as *mut _,
            wrkmem.as_mut_ptr().cast(),
        );

        core::slice::from_raw_parts_mut(output.as_mut_ptr(), out_len)
    }
}

/// Decompress the given `input` to the `output` slice, returning a slice containing the decompressed data.
pub fn decompress_to_slice<'a>(input: &[u8], output: &'a mut [u8]) -> Result<&'a mut [u8]> {
    let mut op = 0;
    let mut ip = 0;
    let mut t;
    let mut next;
    let mut state = 0;
    let mut m_pos;

    if input.len() < 3 {
        return Err(Error::InputOverrun);
    }

    if input[ip] > 17 {
        t = input[ip] as usize - 17;
        ip += 1;

        if t < 4 {
            state = t;
        } else {
            state = 4;
        }

        if output.len() - op < t {
            return Err(Error::Error);
        }

        if input.len() - ip < t + 3 {
            return Err(Error::InputOverrun);
        }

        output[op..op + t].copy_from_slice(&input[ip..ip + t]);
        op += t;
        ip += t;
    }

    loop {
        t = input[ip] as usize;
        ip += 1;

        if t < 16 {
            if state == 0 {
                if t == 0 {
                    let ip_last = ip;

                    while input[ip] == 0 {
                        ip += 1;

                        if input.len() - ip == 0 {
                            return Err(Error::InputOverrun);
                        }
                    }

                    let mut offset = ip - ip_last;

                    if offset > usize::MAX / 255 - 2 {
                        return Err(Error::Error);
                    }

                    offset = (offset << 8) - offset;
                    t += offset + 15 + input[ip] as usize;
                    ip += 1;
                }

                t += 3;

                if output.len() - op < t {
                    return Err(Error::Error);
                }

                if input.len() - ip < t + 3 {
                    return Err(Error::InputOverrun);
                }

                output[op..op + t].copy_from_slice(&input[ip..ip + t]);
                op += t;
                ip += t;

                state = 4;

                continue;
            } else if state != 4 {
                next = t & 3;
                m_pos = op - 1;
                m_pos -= t >> 2;
                m_pos -= (input[ip] as usize) << 2;
                ip += 1;

                // if m_pos < 0 {
                //     current_block = 48;
                //     break;
                // }

                if output.len() - op < 2 {
                    return Err(Error::Error);
                }

                output[op] = output[m_pos];
                output[op + 1] = output[m_pos + 1];
                op += 2;
            } else {
                next = t & 3;
                m_pos = op - (1 + 0x800);
                m_pos -= t >> 2;
                m_pos -= (input[ip] as usize) << 2;
                ip += 1;

                // if m_pos < 0 {
                //     current_block = 48;
                //     break;
                // }

                if output.len() - op < t {
                    return Err(Error::Error);
                }

                for _ in 0..t {
                    output[op] = output[m_pos];
                    m_pos += 1;
                    op += 1;
                }
            }
        } else {
            if t >= 64 {
                next = t & 3;
                m_pos = op - 1;
                m_pos -= t >> 2 & 7;
                m_pos -= (input[ip] as usize) << 3;
                ip += 1;
                t = (t >> 5) - 1 + (3 - 1);
            } else if t >= 32 {
                t = (t & 31) + (3 - 1);

                if t == 2 {
                    let ip_last = ip;

                    while input[ip] == 0 {
                        ip += 1;

                        if input.len() - ip < 1 {
                            return Err(Error::InputOverrun);
                        }
                    }

                    let mut offset = ip - ip_last;

                    if offset > usize::MAX / 255 - 2 {
                        return Err(Error::Error);
                    }

                    offset = (offset << 8) - offset;
                    t += offset + 31 + input[ip] as usize;
                    ip += 1;

                    if input.len() - ip < 2 {
                        return Err(Error::InputOverrun);
                    }
                }

                m_pos = op - 1;
                next = u16::from_le_bytes(input[ip..ip + 2].try_into().unwrap()) as usize;
                ip += 2;
                m_pos -= next >> 2;
                next &= 3;
            } else {
                m_pos = op;
                m_pos -= (t & 8) << 11;
                t = (t & 7) + (3 - 1);

                if t == 2 {
                    let ip_last = ip;

                    while input[ip] == 0 {
                        ip += 1;

                        if input.len() - ip == 0 {
                            return Err(Error::InputOverrun);
                        }
                    }

                    let mut offset = ip - ip_last;

                    if offset > usize::MAX / 255 - 2 {
                        return Err(Error::Error);
                    }

                    offset = (offset << 8) - offset;
                    t += offset + 7 + input[ip] as usize;
                    ip += 1;

                    if input.len() - ip < 2 {
                        return Err(Error::InputOverrun);
                    }
                }

                next = u16::from_le_bytes(input[ip..ip + 2].try_into().unwrap()) as usize;
                ip += 2;
                m_pos -= next >> 2;
                next &= 3;

                if m_pos == op {
                    if t != 3 {
                        return Err(Error::Error);
                    }

                    if ip == input.len() {
                        return Ok(&mut output[..op]);
                    }

                    if ip < input.len() {
                        return Err(Error::InputNotConsumed);
                    }

                    return Err(Error::InputOverrun);
                }

                m_pos -= 0x4000;
            }

            // if m_pos < 0 {
            //     current_block = 48;
            //     break;
            // }

            if output.len() - op < t {
                return Err(Error::Error);
            }

            let offset = op - m_pos;

            if offset < 8 {
                for _ in 0..t {
                    output[op] = output[m_pos];
                    op += 1;
                    m_pos += 1;
                }
            } else {
                let mut count = 0;

                while count + offset < t {
                    output.copy_within(m_pos + count..m_pos + count + offset, op + count);
                    count += offset;
                }

                output.copy_within(m_pos + count..m_pos + t, op + count);
                op += t;
            }
        }

        state = next;

        if input.len() - ip < next + 3 {
            return Err(Error::InputOverrun);
        }

        if output.len() - op < next {
            return Err(Error::Error);
        }

        for _ in 0..next {
            output[op] = input[ip];
            op += 1;
            ip += 1;
        }
    }
}
