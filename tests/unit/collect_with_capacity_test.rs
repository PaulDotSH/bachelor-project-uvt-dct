use dct::collect_with_capacity::CollectWithCapacity;

#[test]
fn test_collect_with_capacity_empty() {
    let empty_iter = std::iter::empty::<i32>();
    let result = empty_iter.collect_with_capacity(10);
    assert_eq!(result.len(), 0);
    assert!(result.capacity() >= 10);
}

#[test]
fn test_collect_with_capacity_smaller_than_capacity() {
    let iter = vec![1, 2, 3].into_iter();
    let result = iter.collect_with_capacity(10);
    assert_eq!(result.len(), 3);
    assert!(result.capacity() >= 10);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_collect_with_capacity_larger_than_capacity() {
    let iter = vec![1, 2, 3, 4, 5].into_iter();
    let result = iter.collect_with_capacity(3);
    assert_eq!(result.len(), 5);
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_collect_with_capacity_exact_capacity() {
    let iter = vec!["a", "b", "c"].into_iter();
    let result = iter.collect_with_capacity(3);
    assert_eq!(result.len(), 3);
    assert!(result.capacity() >= 3);
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_collect_with_capacity_zero_capacity() {
    let iter = vec![1, 2, 3].into_iter();
    let result = iter.collect_with_capacity(0);
    assert_eq!(result.len(), 3);
    assert_eq!(result, vec![1, 2, 3]);
} 