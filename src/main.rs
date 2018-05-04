// Copyright 2016-2018 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate encoding_rs;
extern crate libc;

use encoding_rs::*;

#[link(name = "Kernel32")]
extern "system" {
    fn MultiByteToWideChar(code_page: libc::c_uint,
                           flags: libc::c_ulong,
                           src: *const u8,
                           src_len: libc::c_int,
                           dst: *mut u16,
                           dst_len: libc::c_int)
                           -> libc::c_int;
    // fn WideCharToMultiByte(code_page: libc::c_uint,
    //                        flags: libc::c_ulong,
    //                        src: *const u16,
    //                        src_len: libc::c_int,
    //                        dst: *mut u8,
    //                        dst_len: libc::c_int,
    //                        replacement: *const u8,
    //                        used_replacement: *mut bool)
    //                        -> libc::c_int;
}

static SINGLE_BYTE: [(&'static Encoding, u16); 25] = [
    (&IBM866_INIT, 866),
//    (&ISO_8859_10_INIT, 28600),
    (&ISO_8859_13_INIT, 28603),
//    (&ISO_8859_14_INIT, 28604),
    (&ISO_8859_15_INIT, 28605),
//    (&ISO_8859_16_INIT, 28606),
    (&ISO_8859_2_INIT, 28592),
    (&ISO_8859_3_INIT, 28593),
    (&ISO_8859_4_INIT, 28594),
    (&ISO_8859_5_INIT, 28595),
    (&ISO_8859_6_INIT, 28596),
    (&ISO_8859_7_INIT, 28597),
    (&ISO_8859_8_INIT, 28598),
    (&ISO_8859_8_I_INIT, 38598),
    (&KOI8_R_INIT, 20866),
    (&KOI8_U_INIT, 21866),
    (&MACINTOSH_INIT, 10000),
    (&WINDOWS_1250_INIT, 1250),
    (&WINDOWS_1251_INIT, 1251),
    (&WINDOWS_1252_INIT, 1252),
    (&WINDOWS_1253_INIT, 1253),
    (&WINDOWS_1254_INIT, 1254),
    (&WINDOWS_1255_INIT, 1255),
    (&WINDOWS_1256_INIT, 1256),
    (&WINDOWS_1257_INIT, 1257),
    (&WINDOWS_1258_INIT, 1258),
    (&WINDOWS_874_INIT, 874),
    (&X_MAC_CYRILLIC_INIT, 10017),
];

static TWO_BYTE: [(&'static Encoding, u16); 5] = [
    (&BIG5_INIT, 950),
    (&EUC_KR_INIT, 949),
    (&GBK_INIT, 936),
    (&SHIFT_JIS_INIT, 932),
    (&EUC_JP_INIT, 20932),
];

fn compare_single_byte_encoding(encoding: &'static Encoding, code_page: u16) {
    for b in 0usize..256usize {
        let mut input = [0x20u8; 3];
        input[1] = b as u8;
        let mut output_rs = [0u16; 12];
        let mut output_win32 = [0u16; 12];
        let mut decoder = encoding.new_decoder_without_bom_handling();
        let (result, read, written, had_errors) = decoder.decode_to_utf16(&input[..], &mut output_rs[..], true);
        assert_eq!(result, CoderResult::InputEmpty);
        assert_eq!(read, input.len());
        assert_eq!(written, input.len());
        if had_errors {
            assert_eq!(output_rs[1], 0xFFFD);
        }
        assert_eq!(output_rs[0], 0x20);
        assert_eq!(output_rs[2], 0x20);
        let point_rs = output_rs[1];
        unsafe {
            let written = MultiByteToWideChar(code_page as libc::c_uint, 0, input.as_ptr(), input.len() as libc::c_int, output_win32.as_mut_ptr(), output_win32.len() as libc::c_int);
            assert_eq!(written as usize, input.len());
        }
        assert_eq!(output_win32[0], 0x20);
        assert_eq!(output_win32[2], 0x20);
        let point_win32 = output_win32[1];
        if point_rs != point_win32 {
            println!("Code page {}: {:X} decodes to {:X}; expected {:X}.", code_page, b, point_win32, point_rs);
        }
    }
}

fn compare_two_byte_encoding(encoding: &'static Encoding, code_page: u16) {
    for lead in 0x80usize..256usize {
        for trail in 0usize..256usize {
            let mut input = [0x20u8; 4];
            input[1] = lead as u8;
            input[2] = trail as u8;
            let mut output_rs = [0u16; 12];
            let mut output_win32 = [0u16; 12];
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let (result, read, written_rs, had_errors) = decoder.decode_to_utf16(&input[..], &mut output_rs[..], true);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, input.len());
            assert!(written_rs == 3 || written_rs == 4);
            if had_errors {
                assert_eq!(output_rs[1], 0xFFFD);
            }
            assert_eq!(output_rs[0], 0x20);
            assert_eq!(output_rs[2], 0x20);
            let point_rs = output_rs[1];
            let written_win32 = unsafe {
                MultiByteToWideChar(code_page as libc::c_uint, 0, input.as_ptr(), input.len() as libc::c_int, output_win32.as_mut_ptr(), output_win32.len() as libc::c_int)
            };
            assert_eq!(output_win32[0], 0x20);
            assert_eq!(output_win32[written_win32 as usize - 1], 0x20);
            assert!(written_win32 == 3 || written_win32 == 4);
            let point_win32 = output_win32[1];
            if written_win32 == 3 {
                if written_rs == 3 {
                    if point_rs != point_win32 {
                        println!("Code page {}: {:X} {:X} decodes to {:X}; expected {:X}.", code_page, lead, trail, point_win32, point_rs);
                    }
                } else {
                    println!("Code page {}: {:X} {:X} decodes to {:X}; expected error.", code_page, lead, trail, point_win32);
                }
            } else {
                if written_rs == 4 {
                    if point_rs != point_win32 || output_rs[2] != output_win32[2] {
                        println!("Code page {}: {:X} {:X} decodes to {:X} {:X}; expected {:X} {:X}.", code_page, lead, trail, point_win32, output_win32[2], point_rs, output_rs[2]);
                    }
                } else {
                    println!("Code page {}: {:X} {:X} decodes to error; expected {:X}.", code_page, lead, trail, point_rs);
                }
            }
        }
    }
}

fn main() {
    for &(encoding, code_page) in &SINGLE_BYTE[..] {
        compare_single_byte_encoding(encoding, code_page);
    }
    for &(encoding, code_page) in &TWO_BYTE[..] {
        compare_single_byte_encoding(encoding, code_page);
        compare_two_byte_encoding(encoding, code_page);
    }
}
