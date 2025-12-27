use sig_reactive::*;

#[test]
fn test_provide_and_consume_context() {
    create_scope(move || {
        provide_context(42i32);
        let value: Option<i32> = consume_context();
        assert_eq!(value, Some(42));
    });
}

#[test]
fn test_context_inheritance() {
    create_scope(move || {
        provide_context("outer".to_string());
        
        create_scope(move || {
            let value: Option<String> = consume_context();
            assert_eq!(value, Some("outer".to_string()));
        });
    });
}

#[test]
fn test_context_override() {
    create_scope(move || {
        provide_context(100i32);
        assert_eq!(consume_context::<i32>(), Some(100));
        
        create_scope(move || {
            provide_context(200i32);
            assert_eq!(consume_context::<i32>(), Some(200));
        });
        
        assert_eq!(consume_context::<i32>(), Some(100));
    });
}

#[test]
fn test_multiple_context_types() {
    create_scope(move || {
        provide_context(42i32);
        provide_context("hello".to_string());
        provide_context(3.14f64);
        
        assert_eq!(consume_context::<i32>(), Some(42));
        assert_eq!(consume_context::<String>(), Some("hello".to_string()));
        assert_eq!(consume_context::<f64>(), Some(3.14));
    });
}

#[test]
fn test_context_not_found() {
    create_scope(move || {
        let value: Option<i32> = consume_context();
        assert_eq!(value, None);
    });
}
