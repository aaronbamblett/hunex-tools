//! Real script text files, gated on HUNEX_CTD_DIR (the folder with the four stored .ctd entries).

use std::path::PathBuf;

use hunex_ctd::Text;

const FILES: [(&str, usize, usize, &str); 4] = [
    (
        "script_text_en.ctd",
        1_650_534,
        24_134,
        "8f895ba023ad6232d93d84125595de01a63fe49355b8847598328608a786a690",
    ),
    (
        "script_text_ja.ctd",
        2_017_337,
        24_134,
        "bd74b123e029e23bdb608c7911e5c9f1d1a662d1e9bd54b5672d794b82598ab6",
    ),
    (
        "script_text_zc.ctd",
        1_577_956,
        24_134,
        "f59e27dc9f68f8417e56432d2269343fa1f0db6fff97265c347d2e6eca320f5f",
    ),
    (
        "script_text_zt.ctd",
        1_594_635,
        24_134,
        "d20c2a74c6f23d7a0e2c7b89b491ba281b56eeddc5d0fec05262c94554a73967",
    ),
];

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 of a byte string, used only to check the spec's sample hashes.
fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in message.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (slot, value) in h.iter_mut().zip(v) {
            *slot = slot.wrapping_add(value);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}

#[test]
fn sha256_matches_the_standard_test_vector() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

fn ctd_dir() -> Option<PathBuf> {
    match std::env::var_os("HUNEX_CTD_DIR") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("HUNEX_CTD_DIR is not set; skipping .ctd checks");
            None
        }
    }
}

#[test]
fn bodies_match_the_spec_and_round_trip_byte_for_byte() {
    let Some(dir) = ctd_dir() else {
        return;
    };
    for (name, body_size, lines, sha) in FILES {
        let stored = std::fs::read(dir.join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let body = hunex_lenzu::decompress(&stored).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(body.len(), body_size, "{name}: body size");
        assert_eq!(sha256_hex(&body), sha, "{name}: body SHA-256");
        let text = Text::parse(&body).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(text.len(), lines, "{name}: line count");
        assert!(text.write() == body, "{name}: write differs from the body");
        assert!(
            hunex_lenzu::decompress(&text.to_stored()).unwrap() == body,
            "{name}: literal stored form decodes to a different body"
        );
        assert_eq!(
            Text::from_bytes(&stored).unwrap(),
            text,
            "{name}: stored and body parse differently"
        );
        println!(
            "{name}: {} lines, body {} bytes, sha ok",
            text.len(),
            body.len()
        );
    }
}

#[test]
fn ids_map_to_line_indices() {
    let Some(dir) = ctd_dir() else {
        return;
    };
    let stored = std::fs::read(dir.join("script_text_en.ctd")).unwrap();
    let text = Text::from_bytes(&stored).unwrap();
    assert_eq!(text.lookup("$000000"), text.line(0));
    assert_eq!(text.lookup("$024133"), text.line(24_133));
    assert_eq!(text.lookup("$024134"), None);
}
