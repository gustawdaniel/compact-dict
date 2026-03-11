#![cfg(feature = "rkyv")]

use compact_dict::dict::Dict;
use rkyv::{ser::serializers::AllocSerializer, ser::Serializer};

#[test]
fn test_rkyv_zero_copy_lookup() {
    let mut dict = Dict::<u32>::new(16);
    dict.put("one", 1);
    dict.put("two", 2);
    dict.put("three", 3);
    dict.put("four_super_large_key_just_in_case", 4);

    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&dict).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = unsafe { rkyv::archived_root::<Dict<u32>>(&bytes) };

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
