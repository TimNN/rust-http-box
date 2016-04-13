// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License");                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

use byte::*;

#[test]
#[allow(cyclomatic_complexity)]
fn byte_to_hex_valid() {
    assert_eq!(&byte_to_hex(0x00), b"00");
    assert_eq!(&byte_to_hex(0x01), b"01");
    assert_eq!(&byte_to_hex(0x02), b"02");
    assert_eq!(&byte_to_hex(0x03), b"03");
    assert_eq!(&byte_to_hex(0x04), b"04");
    assert_eq!(&byte_to_hex(0x05), b"05");
    assert_eq!(&byte_to_hex(0x06), b"06");
    assert_eq!(&byte_to_hex(0x07), b"07");
    assert_eq!(&byte_to_hex(0x08), b"08");
    assert_eq!(&byte_to_hex(0x09), b"09");
    assert_eq!(&byte_to_hex(0x0A), b"0A");
    assert_eq!(&byte_to_hex(0x0B), b"0B");
    assert_eq!(&byte_to_hex(0x0C), b"0C");
    assert_eq!(&byte_to_hex(0x0D), b"0D");
    assert_eq!(&byte_to_hex(0x0E), b"0E");
    assert_eq!(&byte_to_hex(0x0F), b"0F");
    assert_eq!(&byte_to_hex(0x10), b"10");
    assert_eq!(&byte_to_hex(0x11), b"11");
    assert_eq!(&byte_to_hex(0x12), b"12");
    assert_eq!(&byte_to_hex(0x13), b"13");
    assert_eq!(&byte_to_hex(0x14), b"14");
    assert_eq!(&byte_to_hex(0x15), b"15");
    assert_eq!(&byte_to_hex(0x16), b"16");
    assert_eq!(&byte_to_hex(0x17), b"17");
    assert_eq!(&byte_to_hex(0x18), b"18");
    assert_eq!(&byte_to_hex(0x19), b"19");
    assert_eq!(&byte_to_hex(0x1A), b"1A");
    assert_eq!(&byte_to_hex(0x1B), b"1B");
    assert_eq!(&byte_to_hex(0x1C), b"1C");
    assert_eq!(&byte_to_hex(0x1D), b"1D");
    assert_eq!(&byte_to_hex(0x1E), b"1E");
    assert_eq!(&byte_to_hex(0x1F), b"1F");
    assert_eq!(&byte_to_hex(0x20), b"20");
    assert_eq!(&byte_to_hex(0x21), b"21");
    assert_eq!(&byte_to_hex(0x22), b"22");
    assert_eq!(&byte_to_hex(0x23), b"23");
    assert_eq!(&byte_to_hex(0x24), b"24");
    assert_eq!(&byte_to_hex(0x25), b"25");
    assert_eq!(&byte_to_hex(0x26), b"26");
    assert_eq!(&byte_to_hex(0x27), b"27");
    assert_eq!(&byte_to_hex(0x28), b"28");
    assert_eq!(&byte_to_hex(0x29), b"29");
    assert_eq!(&byte_to_hex(0x2A), b"2A");
    assert_eq!(&byte_to_hex(0x2B), b"2B");
    assert_eq!(&byte_to_hex(0x2C), b"2C");
    assert_eq!(&byte_to_hex(0x2D), b"2D");
    assert_eq!(&byte_to_hex(0x2E), b"2E");
    assert_eq!(&byte_to_hex(0x2F), b"2F");
    assert_eq!(&byte_to_hex(0x30), b"30");
    assert_eq!(&byte_to_hex(0x31), b"31");
    assert_eq!(&byte_to_hex(0x32), b"32");
    assert_eq!(&byte_to_hex(0x33), b"33");
    assert_eq!(&byte_to_hex(0x34), b"34");
    assert_eq!(&byte_to_hex(0x35), b"35");
    assert_eq!(&byte_to_hex(0x36), b"36");
    assert_eq!(&byte_to_hex(0x37), b"37");
    assert_eq!(&byte_to_hex(0x38), b"38");
    assert_eq!(&byte_to_hex(0x39), b"39");
    assert_eq!(&byte_to_hex(0x3A), b"3A");
    assert_eq!(&byte_to_hex(0x3B), b"3B");
    assert_eq!(&byte_to_hex(0x3C), b"3C");
    assert_eq!(&byte_to_hex(0x3D), b"3D");
    assert_eq!(&byte_to_hex(0x3E), b"3E");
    assert_eq!(&byte_to_hex(0x3F), b"3F");
    assert_eq!(&byte_to_hex(0x40), b"40");
    assert_eq!(&byte_to_hex(0x41), b"41");
    assert_eq!(&byte_to_hex(0x42), b"42");
    assert_eq!(&byte_to_hex(0x43), b"43");
    assert_eq!(&byte_to_hex(0x44), b"44");
    assert_eq!(&byte_to_hex(0x45), b"45");
    assert_eq!(&byte_to_hex(0x46), b"46");
    assert_eq!(&byte_to_hex(0x47), b"47");
    assert_eq!(&byte_to_hex(0x48), b"48");
    assert_eq!(&byte_to_hex(0x49), b"49");
    assert_eq!(&byte_to_hex(0x4A), b"4A");
    assert_eq!(&byte_to_hex(0x4B), b"4B");
    assert_eq!(&byte_to_hex(0x4C), b"4C");
    assert_eq!(&byte_to_hex(0x4D), b"4D");
    assert_eq!(&byte_to_hex(0x4E), b"4E");
    assert_eq!(&byte_to_hex(0x4F), b"4F");
    assert_eq!(&byte_to_hex(0x50), b"50");
    assert_eq!(&byte_to_hex(0x51), b"51");
    assert_eq!(&byte_to_hex(0x52), b"52");
    assert_eq!(&byte_to_hex(0x53), b"53");
    assert_eq!(&byte_to_hex(0x54), b"54");
    assert_eq!(&byte_to_hex(0x55), b"55");
    assert_eq!(&byte_to_hex(0x56), b"56");
    assert_eq!(&byte_to_hex(0x57), b"57");
    assert_eq!(&byte_to_hex(0x58), b"58");
    assert_eq!(&byte_to_hex(0x59), b"59");
    assert_eq!(&byte_to_hex(0x5A), b"5A");
    assert_eq!(&byte_to_hex(0x5B), b"5B");
    assert_eq!(&byte_to_hex(0x5C), b"5C");
    assert_eq!(&byte_to_hex(0x5D), b"5D");
    assert_eq!(&byte_to_hex(0x5E), b"5E");
    assert_eq!(&byte_to_hex(0x5F), b"5F");
    assert_eq!(&byte_to_hex(0x60), b"60");
    assert_eq!(&byte_to_hex(0x61), b"61");
    assert_eq!(&byte_to_hex(0x62), b"62");
    assert_eq!(&byte_to_hex(0x63), b"63");
    assert_eq!(&byte_to_hex(0x64), b"64");
    assert_eq!(&byte_to_hex(0x65), b"65");
    assert_eq!(&byte_to_hex(0x66), b"66");
    assert_eq!(&byte_to_hex(0x67), b"67");
    assert_eq!(&byte_to_hex(0x68), b"68");
    assert_eq!(&byte_to_hex(0x69), b"69");
    assert_eq!(&byte_to_hex(0x6A), b"6A");
    assert_eq!(&byte_to_hex(0x6B), b"6B");
    assert_eq!(&byte_to_hex(0x6C), b"6C");
    assert_eq!(&byte_to_hex(0x6D), b"6D");
    assert_eq!(&byte_to_hex(0x6E), b"6E");
    assert_eq!(&byte_to_hex(0x6F), b"6F");
    assert_eq!(&byte_to_hex(0x70), b"70");
    assert_eq!(&byte_to_hex(0x71), b"71");
    assert_eq!(&byte_to_hex(0x72), b"72");
    assert_eq!(&byte_to_hex(0x73), b"73");
    assert_eq!(&byte_to_hex(0x74), b"74");
    assert_eq!(&byte_to_hex(0x75), b"75");
    assert_eq!(&byte_to_hex(0x76), b"76");
    assert_eq!(&byte_to_hex(0x77), b"77");
    assert_eq!(&byte_to_hex(0x78), b"78");
    assert_eq!(&byte_to_hex(0x79), b"79");
    assert_eq!(&byte_to_hex(0x7A), b"7A");
    assert_eq!(&byte_to_hex(0x7B), b"7B");
    assert_eq!(&byte_to_hex(0x7C), b"7C");
    assert_eq!(&byte_to_hex(0x7D), b"7D");
    assert_eq!(&byte_to_hex(0x7E), b"7E");
    assert_eq!(&byte_to_hex(0x7F), b"7F");
    assert_eq!(&byte_to_hex(0x80), b"80");
    assert_eq!(&byte_to_hex(0x81), b"81");
    assert_eq!(&byte_to_hex(0x82), b"82");
    assert_eq!(&byte_to_hex(0x83), b"83");
    assert_eq!(&byte_to_hex(0x84), b"84");
    assert_eq!(&byte_to_hex(0x85), b"85");
    assert_eq!(&byte_to_hex(0x86), b"86");
    assert_eq!(&byte_to_hex(0x87), b"87");
    assert_eq!(&byte_to_hex(0x88), b"88");
    assert_eq!(&byte_to_hex(0x89), b"89");
    assert_eq!(&byte_to_hex(0x8A), b"8A");
    assert_eq!(&byte_to_hex(0x8B), b"8B");
    assert_eq!(&byte_to_hex(0x8C), b"8C");
    assert_eq!(&byte_to_hex(0x8D), b"8D");
    assert_eq!(&byte_to_hex(0x8E), b"8E");
    assert_eq!(&byte_to_hex(0x8F), b"8F");
    assert_eq!(&byte_to_hex(0x90), b"90");
    assert_eq!(&byte_to_hex(0x91), b"91");
    assert_eq!(&byte_to_hex(0x92), b"92");
    assert_eq!(&byte_to_hex(0x93), b"93");
    assert_eq!(&byte_to_hex(0x94), b"94");
    assert_eq!(&byte_to_hex(0x95), b"95");
    assert_eq!(&byte_to_hex(0x96), b"96");
    assert_eq!(&byte_to_hex(0x97), b"97");
    assert_eq!(&byte_to_hex(0x98), b"98");
    assert_eq!(&byte_to_hex(0x99), b"99");
    assert_eq!(&byte_to_hex(0x9A), b"9A");
    assert_eq!(&byte_to_hex(0x9B), b"9B");
    assert_eq!(&byte_to_hex(0x9C), b"9C");
    assert_eq!(&byte_to_hex(0x9D), b"9D");
    assert_eq!(&byte_to_hex(0x9E), b"9E");
    assert_eq!(&byte_to_hex(0x9F), b"9F");
    assert_eq!(&byte_to_hex(0xA0), b"A0");
    assert_eq!(&byte_to_hex(0xA1), b"A1");
    assert_eq!(&byte_to_hex(0xA2), b"A2");
    assert_eq!(&byte_to_hex(0xA3), b"A3");
    assert_eq!(&byte_to_hex(0xA4), b"A4");
    assert_eq!(&byte_to_hex(0xA5), b"A5");
    assert_eq!(&byte_to_hex(0xA6), b"A6");
    assert_eq!(&byte_to_hex(0xA7), b"A7");
    assert_eq!(&byte_to_hex(0xA8), b"A8");
    assert_eq!(&byte_to_hex(0xA9), b"A9");
    assert_eq!(&byte_to_hex(0xAA), b"AA");
    assert_eq!(&byte_to_hex(0xAB), b"AB");
    assert_eq!(&byte_to_hex(0xAC), b"AC");
    assert_eq!(&byte_to_hex(0xAD), b"AD");
    assert_eq!(&byte_to_hex(0xAE), b"AE");
    assert_eq!(&byte_to_hex(0xAF), b"AF");
    assert_eq!(&byte_to_hex(0xB0), b"B0");
    assert_eq!(&byte_to_hex(0xB1), b"B1");
    assert_eq!(&byte_to_hex(0xB2), b"B2");
    assert_eq!(&byte_to_hex(0xB3), b"B3");
    assert_eq!(&byte_to_hex(0xB4), b"B4");
    assert_eq!(&byte_to_hex(0xB5), b"B5");
    assert_eq!(&byte_to_hex(0xB6), b"B6");
    assert_eq!(&byte_to_hex(0xB7), b"B7");
    assert_eq!(&byte_to_hex(0xB8), b"B8");
    assert_eq!(&byte_to_hex(0xB9), b"B9");
    assert_eq!(&byte_to_hex(0xBA), b"BA");
    assert_eq!(&byte_to_hex(0xBB), b"BB");
    assert_eq!(&byte_to_hex(0xBC), b"BC");
    assert_eq!(&byte_to_hex(0xBD), b"BD");
    assert_eq!(&byte_to_hex(0xBE), b"BE");
    assert_eq!(&byte_to_hex(0xBF), b"BF");
    assert_eq!(&byte_to_hex(0xC0), b"C0");
    assert_eq!(&byte_to_hex(0xC1), b"C1");
    assert_eq!(&byte_to_hex(0xC2), b"C2");
    assert_eq!(&byte_to_hex(0xC3), b"C3");
    assert_eq!(&byte_to_hex(0xC4), b"C4");
    assert_eq!(&byte_to_hex(0xC5), b"C5");
    assert_eq!(&byte_to_hex(0xC6), b"C6");
    assert_eq!(&byte_to_hex(0xC7), b"C7");
    assert_eq!(&byte_to_hex(0xC8), b"C8");
    assert_eq!(&byte_to_hex(0xC9), b"C9");
    assert_eq!(&byte_to_hex(0xCA), b"CA");
    assert_eq!(&byte_to_hex(0xCB), b"CB");
    assert_eq!(&byte_to_hex(0xCC), b"CC");
    assert_eq!(&byte_to_hex(0xCD), b"CD");
    assert_eq!(&byte_to_hex(0xCE), b"CE");
    assert_eq!(&byte_to_hex(0xCF), b"CF");
    assert_eq!(&byte_to_hex(0xD0), b"D0");
    assert_eq!(&byte_to_hex(0xD1), b"D1");
    assert_eq!(&byte_to_hex(0xD2), b"D2");
    assert_eq!(&byte_to_hex(0xD3), b"D3");
    assert_eq!(&byte_to_hex(0xD4), b"D4");
    assert_eq!(&byte_to_hex(0xD5), b"D5");
    assert_eq!(&byte_to_hex(0xD6), b"D6");
    assert_eq!(&byte_to_hex(0xD7), b"D7");
    assert_eq!(&byte_to_hex(0xD8), b"D8");
    assert_eq!(&byte_to_hex(0xD9), b"D9");
    assert_eq!(&byte_to_hex(0xDA), b"DA");
    assert_eq!(&byte_to_hex(0xDB), b"DB");
    assert_eq!(&byte_to_hex(0xDC), b"DC");
    assert_eq!(&byte_to_hex(0xDD), b"DD");
    assert_eq!(&byte_to_hex(0xDE), b"DE");
    assert_eq!(&byte_to_hex(0xDF), b"DF");
    assert_eq!(&byte_to_hex(0xE0), b"E0");
    assert_eq!(&byte_to_hex(0xE1), b"E1");
    assert_eq!(&byte_to_hex(0xE2), b"E2");
    assert_eq!(&byte_to_hex(0xE3), b"E3");
    assert_eq!(&byte_to_hex(0xE4), b"E4");
    assert_eq!(&byte_to_hex(0xE5), b"E5");
    assert_eq!(&byte_to_hex(0xE6), b"E6");
    assert_eq!(&byte_to_hex(0xE7), b"E7");
    assert_eq!(&byte_to_hex(0xE8), b"E8");
    assert_eq!(&byte_to_hex(0xE9), b"E9");
    assert_eq!(&byte_to_hex(0xEA), b"EA");
    assert_eq!(&byte_to_hex(0xEB), b"EB");
    assert_eq!(&byte_to_hex(0xEC), b"EC");
    assert_eq!(&byte_to_hex(0xED), b"ED");
    assert_eq!(&byte_to_hex(0xEE), b"EE");
    assert_eq!(&byte_to_hex(0xEF), b"EF");
    assert_eq!(&byte_to_hex(0xF0), b"F0");
    assert_eq!(&byte_to_hex(0xF1), b"F1");
    assert_eq!(&byte_to_hex(0xF2), b"F2");
    assert_eq!(&byte_to_hex(0xF3), b"F3");
    assert_eq!(&byte_to_hex(0xF4), b"F4");
    assert_eq!(&byte_to_hex(0xF5), b"F5");
    assert_eq!(&byte_to_hex(0xF6), b"F6");
    assert_eq!(&byte_to_hex(0xF7), b"F7");
    assert_eq!(&byte_to_hex(0xF8), b"F8");
    assert_eq!(&byte_to_hex(0xF9), b"F9");
    assert_eq!(&byte_to_hex(0xFA), b"FA");
    assert_eq!(&byte_to_hex(0xFB), b"FB");
    assert_eq!(&byte_to_hex(0xFC), b"FC");
    assert_eq!(&byte_to_hex(0xFD), b"FD");
    assert_eq!(&byte_to_hex(0xFE), b"FE");
    assert_eq!(&byte_to_hex(0xFF), b"FF");
}
