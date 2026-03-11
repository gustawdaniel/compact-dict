import sys

with open('src/dict.rs', 'r') as f:
    content = f.read()

start_idx = content.find("    pub fn put(&mut self, key: &str, value: V) {")
end_idx = content.find("    pub fn get_or(&self, key: &str, default: V) -> V {")

if start_idx != -1 and end_idx != -1:
    replacement = """    pub fn put(&mut self, key: &str, value: V) {
        self.maybe_rehash();

        let existing_idx = self.find_key_index(key);
        if existing_idx != 0 {
            let idx0 = existing_idx - 1;
            self.values[idx0] = value;
            if DESTRUCTIVE && self.is_deleted(idx0) {
                self.count += 1;
                self.clear_deleted(idx0);
            }
            return;
        }

        self.keys.add(key);
        self.values.push(value);
        let mut current_key_index = self.keys.len(); 
        let key_hash_u64 = self.hasher.hash(key);
        let mut current_key_hash =
            (key_hash_u64 as usize) & ((1 << (std::mem::size_of::<KC>() * 8)) - 1);

        let modulo_mask = self.capacity - 1;
        let mut slot = current_key_hash & modulo_mask;
        let mut current_dib = 0;

        loop {
            let slot_key_index = self.load_slot(slot);
            if slot_key_index == 0 {
                self.store_slot(slot, current_key_index);
                if CACHING_HASHES {
                    self.key_hashes.as_mut().unwrap()[slot] = KC::try_from(current_key_hash)
                        .ok()
                        .expect("Conversion failed");
                }
                self.count += 1;
                return;
            }

            if CACHING_HASHES {
                let slot_hash_val: usize = self.key_hashes.as_ref().unwrap()[slot]
                    .try_into()
                    .ok()
                    .expect("Conversion failed");
                let slot_original = slot_hash_val & modulo_mask;
                let slot_dib = (slot + self.capacity - slot_original) & modulo_mask;

                if current_dib > slot_dib {
                    self.store_slot(slot, current_key_index);
                    current_key_index = slot_key_index;
                    
                    self.key_hashes.as_mut().unwrap()[slot] = KC::try_from(current_key_hash)
                        .ok()
                        .expect("Conversion failed");
                    current_key_hash = slot_hash_val;
                    
                    current_dib = slot_dib;
                }
            }

            slot = (slot + 1) & modulo_mask;
            current_dib += 1;
        }
    }

"""
    new_content = content[:start_idx] + replacement + content[end_idx:]
    with open('src/dict.rs', 'w') as f:
        f.write(new_content)
    print("REPLACED SUCCESSFULLY")
else:
    print("COULD NOT FIND INDICES")
