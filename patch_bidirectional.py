import sys

with open('src/dict.rs', 'r') as f:
    content = f.read()

# Replace find_key_index
start_idx = content.find("#[inline]\n    fn find_key_index(&self, key: &str) -> usize {")
end_idx = content.find("    pub fn put(&mut self, key: &str, value: V) {")

if start_idx != -1 and end_idx != -1:
    replacement_find = """#[inline]
    fn find_key_index(&self, key: &str) -> usize {
        let key_hash_u64 = self.hasher.hash(key);
        let key_hash_truncated = (key_hash_u64 as usize) & ((1 << (std::mem::size_of::<KC>() * 8)) - 1);
        let modulo_mask = self.capacity - 1;
        
        let initial_slot = key_hash_truncated & modulo_mask;
        let mut slot = initial_slot;
        
        let mut probe_step = 1;
        let mut is_pos = true;

        loop {
            let key_index = self.load_slot(slot);
            if key_index == 0 {
                return 0;
            }

            if CACHING_HASHES {
                let other_hash: usize = self.key_hashes.as_ref().unwrap()[slot]
                    .try_into()
                    .ok()
                    .expect("KC -> usize conversion failed");
                if other_hash == key_hash_truncated {
                    let other_key = self.keys.get(key_index - 1).unwrap();
                    if other_key == key {
                        return key_index;
                    }
                }
            } else {
                let other_key = self.keys.get(key_index - 1).unwrap();
                if other_key == key {
                    return key_index;
                }
            }

            if is_pos {
                slot = (initial_slot + probe_step) & modulo_mask;
                is_pos = false;
            } else {
                slot = (initial_slot + self.capacity - probe_step) & modulo_mask;
                probe_step += 1;
                is_pos = true;
            }
        }
    }

"""
    content = content[:start_idx] + replacement_find + content[end_idx:]
else:
    print("FAILED TO FIND find_key_index")

# Replace put
start_idx = content.find("    pub fn put(&mut self, key: &str, value: V) {")
end_idx = content.find("    pub fn get_or(&self, key: &str, default: V) -> V {")

if start_idx != -1 and end_idx != -1:
    replacement_put = """    pub fn put(&mut self, key: &str, value: V) {
        self.maybe_rehash();

        let key_hash_u64 = self.hasher.hash(key);
        let key_hash_truncated =
            (key_hash_u64 as usize) & ((1 << (std::mem::size_of::<KC>() * 8)) - 1);

        let modulo_mask = self.capacity - 1;
        let initial_slot = key_hash_truncated & modulo_mask;
        let mut slot = initial_slot;
        
        let mut probe_step = 1;
        let mut is_pos = true;

        loop {
            let key_index = self.load_slot(slot);
            if key_index == 0 {
                self.keys.add(key);
                if CACHING_HASHES {
                    self.key_hashes.as_mut().unwrap()[slot] = KC::try_from(key_hash_truncated)
                        .ok()
                        .expect("conversion failed");
                }
                self.values.push(value);
                self.store_slot(slot, self.keys.len());
                self.count += 1;
                return;
            }

            if CACHING_HASHES {
                let other_hash: usize = self.key_hashes.as_ref().unwrap()[slot]
                    .try_into()
                    .ok()
                    .expect("conversion failed");
                if other_hash == key_hash_truncated {
                    let other_key = self.keys.get(key_index - 1).unwrap();
                    if other_key == key {
                        let idx0 = key_index - 1;
                        self.values[idx0] = value;
                        if DESTRUCTIVE && self.is_deleted(idx0) {
                            self.count += 1;
                            self.clear_deleted(idx0);
                        }
                        return;
                    }
                }
            } else {
                let other_key = self.keys.get(key_index - 1).unwrap();
                if other_key == key {
                    let idx0 = key_index - 1;
                    self.values[idx0] = value;
                    if DESTRUCTIVE && self.is_deleted(idx0) {
                        self.count += 1;
                        self.clear_deleted(idx0);
                    }
                    return;
                }
            }

            if is_pos {
                slot = (initial_slot + probe_step) & modulo_mask;
                is_pos = false;
            } else {
                slot = (initial_slot + self.capacity - probe_step) & modulo_mask;
                probe_step += 1;
                is_pos = true;
            }
        }
    }

"""
    new_content = content[:start_idx] + replacement_put + content[end_idx:]
    with open('src/dict.rs', 'w') as f:
        f.write(new_content)
    print("REPLACED SUCCESSFULLY")
else:
    print("FAILED TO FIND put")
