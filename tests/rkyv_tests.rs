#![cfg(feature = "rkyv")]

use compact_dict::dict::Dict;
use rkyv::{ser::serializers::AllocSerializer, ser::Serializer};

#[test]
fn test_rkyv_zero_copy_lookup() {
    // Initialize a dictionary and populate it with data
    let mut dict = Dict::<u32>::new(16);
    dict.put("one", 1);
    dict.put("two", 2);
    dict.put("three", 3);
    dict.put("four_super_large_key_just_in_case", 4);

    // Serialize the dictionary into a byte buffer using rkyv
    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&dict).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    // Map the archived version directly from the byte buffer (zero-copy)
    let archived = unsafe { rkyv::archived_root::<Dict<u32>>(&bytes) };

    // Verify that lookups work correctly on the archived structure
    assert_eq!(archived.contains("one"), true);
    assert_eq!(archived.get("one"), Some(1));

    assert_eq!(archived.contains("two"), true);
    assert_eq!(archived.get("two"), Some(2));

    assert_eq!(archived.contains("three"), true);
    assert_eq!(archived.get("three"), Some(3));

    assert_eq!(archived.contains("four_super_large_key_just_in_case"), true);
    assert_eq!(archived.get("four_super_large_key_just_in_case"), Some(4));

    assert_eq!(archived.contains("five"), false);
    assert_eq!(archived.get("five"), None);
}

#[test]
fn test_rkyv_robustness() {
    let mut dict = Dict::<u32>::new(16);
    dict.put("rust", 2026);

    // Serialize to bytes
    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&dict).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    // Simulate mmap by cloning bytes to a new memory location
    let map_simulation = bytes.clone(); 
    
    // Explicitly drop the original dictionary and the intermediate buffer
    // This ensures we are not accidentally relying on the original allocations
    drop(dict);
    drop(bytes);

    // Access the data from the new location. 
    // If this works, it proves the structure uses relative offsets (position-independent)
    let archived = unsafe { rkyv::archived_root::<Dict<u32>>(&map_simulation) };

    // Verify the data is still accessible and correct
    assert_eq!(archived.get("rust"), Some(2026));
}