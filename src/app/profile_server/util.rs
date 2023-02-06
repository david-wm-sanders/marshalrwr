use std::hash::Hasher;

// # by Xe-No
// def get_file_hash(name):
//     lst = [ord(x) for x in name]
//     d1 = 0
//     lst.reverse()
//     for i, element in enumerate(lst):
//         d1 += element * (33**i)
//     d2 = 33**len(lst) * 5381
//     return (d1+d2) % (2**32)

// derived from a python implementation^ by Xe-No
// this seems to overcomplicate things a bit (and won't work w/o big integers),
// a search for `"5381" "33" hash` indicated that this could be djb2 (http://www.cse.yorku.ca/~oz/hash.html)
// a Djb2a hasher was thus devised (with inspiration from some crates that provide various versions of djb2 hash implementations)
// in this implementation, even though the server is x64, we ensure to use u32 (as the rwr client is 32-bit)
pub fn rwr1_hash_username(username: &str) -> u64 {
    let mut djb2a_hasher = Djb2aHash32::default();
    djb2a_hasher.write(username.as_bytes());
    djb2a_hasher.finish()
}

struct Djb2aHash32 {
    v: u32
}

impl Default for Djb2aHash32 {
    fn default() -> Self {
        Self { v: 5381 }
    }
}

impl Hasher for Djb2aHash32 {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.v = (self.v << 5)
                     .wrapping_add(self.v)
                     .wrapping_add(b as u32);
        }
    }

    fn finish(&self) -> u64 {
        self.v as u64
    }
}