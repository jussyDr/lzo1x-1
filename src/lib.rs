//! Safe Rust port of the LZO1X-1 compression algorithm.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt::{self, Debug, Display};

/// Computes the worst case compressed size for the given input `size`.
pub fn worst_compress(size: usize) -> usize {
    size + (size / 16) + 64 + 3
}

/// Decompression error type.
#[derive(Debug)]
pub enum Error {
    /// Input does not have the correct format.
    Format,
    /// Reached end of input buffer but expected more input.
    InputOverrun,
    /// Output buffer was not large enough.
    OutputOverrun,
    /// Input buffer was not entirely consumed.
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
    let mut wrkmem = [0u16; u16::MAX as usize];

    let mut op = 0;
    let mut ip = 0;
    let mut l = input.len();
    let mut t = 0;

    loop {
        if l <= 20 {
            break;
        }

        let ll = if l <= 0xbfff + 1 { l } else { 0xbfff + 1 };
        let ll_end = ip + ll;

        if ll_end + ((t + ll) >> 5) <= ll_end {
            break;
        }

        wrkmem[..8192].fill(0);

        t = {
            let in_ = ip;
            let in_len = ll;
            let mut ti = t;

            let mut current_block;
            let mut ip = in_;
            let in_end = in_ + in_len;
            let ip_end = in_ + in_len - 20;
            let mut ii = in_;

            ip += if ti < 4 { 4 - ti } else { 0 };

            let mut m_off: usize;

            'loop2: loop {
                ip += ((ip - ii) >> 5) + 1;

                loop {
                    if ip >= ip_end {
                        break 'loop2;
                    }

                    let dv = u32::from_le_bytes(input[ip..ip + 4].try_into().unwrap());
                    let mut t =
                        (dv.wrapping_mul(0x1824429D) >> (32 - 13) & ((1 << 13) - 1)) as usize;
                    let m_pos = in_ + wrkmem[t] as usize;
                    wrkmem[t] = (ip - in_) as u16;

                    if dv != u32::from_le_bytes(input[m_pos..m_pos + 4].try_into().unwrap()) {
                        break;
                    }

                    ii -= ti;
                    ti = 0;
                    t = ip - ii;

                    if t != 0 {
                        if t <= 3 {
                            output[op - 2] |= t as u8;
                            output[op] = input[ii];
                            output[op + 1] = input[ii + 1];
                            output[op + 2] = input[ii + 2];
                            output[op + 3] = input[ii + 3];
                            op += t;
                        } else if t <= 16 {
                            output[op] = t as u8 - 3;
                            op += 1;
                            output[op..op + 16].copy_from_slice(&input[ii..ii + 16]);
                            op += t;
                        } else {
                            if t <= 18 {
                                output[op] = t as u8 - 3;
                                op += 1;
                            } else {
                                let mut tt = t - 18;
                                output[op] = 0;
                                op += 1;

                                loop {
                                    if tt <= 255 {
                                        break;
                                    }

                                    tt -= 255;
                                    output[op] = 0;
                                    op += 1;
                                }

                                output[op] = tt as u8;
                                op += 1;
                            }

                            output[op..op + t].copy_from_slice(&input[ii..ii + t]);
                            op += t;
                        }
                    }

                    let mut m_len = 4;

                    if input[ip + m_len] == input[m_pos + m_len] {
                        current_block = 22;
                    } else {
                        current_block = 31;
                    }

                    'loop1: loop {
                        if current_block == 22 {
                            for _ in 0..7 {
                                m_len += 1;

                                if input[ip + m_len] != input[m_pos + m_len] {
                                    current_block = 31;
                                    continue 'loop1;
                                }
                            }

                            m_len += 1;

                            if ip + m_len >= ip_end {
                                current_block = 31;
                                continue;
                            }

                            if input[ip + m_len] as i32 == input[m_pos + m_len] as i32 {
                                current_block = 22;
                            } else {
                                current_block = 31;
                            }
                        } else {
                            m_off = ip - m_pos;
                            ip += m_len;
                            ii = ip;

                            if m_len <= 8 && m_off <= 0x800 {
                                current_block = 47;
                                break;
                            } else {
                                current_block = 32;
                                break;
                            }
                        }
                    }
                    if current_block == 32 {
                        if m_off <= 0x4000 {
                            m_off -= 1;

                            if m_len <= 33 {
                                output[op] = (32 | (m_len - 2)) as u8;
                                op += 1;
                            } else {
                                m_len -= 33;
                                output[op] = 32;
                                op += 1;

                                loop {
                                    if m_len <= 255 {
                                        break;
                                    }

                                    m_len -= 255;
                                    output[op] = 0;
                                    op += 1;
                                }

                                output[op] = m_len as u8;
                                op += 1;
                            }

                            output[op] = (m_off << 2) as u8;
                            op += 1;
                            output[op] = (m_off >> 6) as u8;
                            op += 1;
                        } else {
                            m_off -= 0x4000;

                            if m_len <= 9 {
                                output[op] = (16 | m_off >> 11 & 8 | (m_len - 2)) as u8;
                                op += 1;
                            } else {
                                m_len -= 9;
                                output[op] = (16 | m_off >> 11 & 8) as u8;
                                op += 1;

                                loop {
                                    if m_len <= 255 {
                                        break;
                                    }

                                    m_len -= 255;
                                    output[op] = 0;
                                    op += 1;
                                }

                                output[op] = m_len as u8;
                                op += 1;
                            }

                            output[op] = (m_off << 2) as u8;
                            op += 1;
                            output[op] = (m_off >> 6) as u8;
                            op += 1;
                        }
                    } else {
                        m_off -= 1;
                        output[op] = ((m_len - 1) << 5 | (m_off & 7) << 2) as u8;
                        op += 1;
                        output[op] = (m_off >> 3) as u8;
                        op += 1;
                    }
                }
            }

            in_end - (ii - ti)
        };

        ip += ll;
        l -= ll;
    }

    t += l;

    if t > 0 {
        let ii = input.len() - t;

        if op == 0 && t <= 238 {
            output[op] = t as u8 + 17;
            op += 1;
        } else if t <= 3 {
            output[op - 2] |= t as u8;
        } else if t <= 18 {
            output[op] = t as u8 - 3;
            op += 1;
        } else {
            let mut tt = t - 18;

            output[op] = 0;
            op += 1;

            loop {
                if tt <= 255 {
                    break;
                }

                tt -= 255;
                output[op] = 0;
                op += 1;
            }

            output[op] = tt as u8;
            op += 1;
        }

        output[op..op + t].copy_from_slice(&input[ii..ii + t]);
        op += t;
    }

    output[op] = 17;
    op += 1;
    output[op] = 0;
    op += 1;
    output[op] = 0;
    op += 1;

    &mut output[..op]
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
            return Err(Error::OutputOverrun);
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
                        return Err(Error::Format);
                    }

                    offset = (offset << 8) - offset;
                    t += offset + 15 + input[ip] as usize;
                    ip += 1;
                }

                t += 3;

                if output.len() - op < t {
                    return Err(Error::OutputOverrun);
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
                    return Err(Error::OutputOverrun);
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
                    return Err(Error::OutputOverrun);
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
                        return Err(Error::Format);
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
                        return Err(Error::Format);
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
                        return Err(Error::Format);
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
                return Err(Error::OutputOverrun);
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
            return Err(Error::OutputOverrun);
        }

        for _ in 0..next {
            output[op] = input[ip];
            op += 1;
            ip += 1;
        }
    }
}
