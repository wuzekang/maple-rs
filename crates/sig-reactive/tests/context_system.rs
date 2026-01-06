use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Context System Tests - 完整覆盖
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
struct AppConfig {
    name: String,
    version: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Theme {
    primary_color: String,
    font_size: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct UserSession {
    user_id: u64,
    token: String,
}

// ============================================================================
// 基础 Context 测试
// ============================================================================

#[test]
fn test_provide_and_consume_context() {
    create_scope(|| {
        // Provide context
        let config = AppConfig {
            name: "MyApp".to_string(),
            version: 1,
        };
        provide_context(config.clone());

        // Consume context
        let retrieved = consume_context::<AppConfig>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), config);
    });
}

#[test]
fn test_context_not_found() {
    create_scope(|| {
        // Try to consume context that doesn't exist
        let result = consume_context::<AppConfig>();
        assert!(result.is_none());
    });
}

#[test]
fn test_has_context() {
    create_scope(|| {
        // Initially no context
        assert!(!has_context::<AppConfig>());

        // Provide context
        provide_context(AppConfig {
            name: "Test".to_string(),
            version: 1,
        });

        // Now has context
        assert!(has_context::<AppConfig>());
    });
}

#[test]
fn test_context_override() {
    create_scope(|| {
        // First provide
        provide_context(Theme {
            primary_color: "red".to_string(),
            font_size: 16,
        });

        let theme1 = consume_context::<Theme>();
        assert_eq!(theme1.as_ref().unwrap().primary_color, "red");

        // Override with new value
        provide_context(Theme {
            primary_color: "blue".to_string(),
            font_size: 18,
        });

        let theme2 = consume_context::<Theme>();
        assert_eq!(theme2.as_ref().unwrap().primary_color, "blue");
        assert_eq!(theme2.as_ref().unwrap().font_size, 18);
    });
}

// ============================================================================
// Context 层次结构测试
// ============================================================================

#[test]
fn test_context_hierarchy() {
    create_scope(|| {
        // Parent scope provides context
        provide_context(AppConfig {
            name: "Parent".to_string(),
            version: 1,
        });

        // Child scope can consume parent's context
        create_scope(|| {
            let config = consume_context::<AppConfig>();
            assert!(config.is_some());
            assert_eq!(config.unwrap().name, "Parent");
        });
    });
}

#[test]
fn test_nested_context_hierarchy() {
    create_scope(|| {
        // Level 1
        provide_context(AppConfig {
            name: "Level1".to_string(),
            version: 1,
        });

        create_scope(|| {
            // Level 2
            provide_context(Theme {
                primary_color: "red".to_string(),
                font_size: 16,
            });

            create_scope(|| {
                // Level 3 - can access both contexts
                let config = consume_context::<AppConfig>();
                let theme = consume_context::<Theme>();

                assert!(config.is_some());
                assert!(theme.is_some());
                assert_eq!(config.unwrap().name, "Level1");
                assert_eq!(theme.unwrap().primary_color, "red");
            });
        });
    });
}

#[test]
fn test_context_shadowing() {
    create_scope(|| {
        // Parent provides config
        provide_context(AppConfig {
            name: "Parent".to_string(),
            version: 1,
        });

        create_scope(|| {
            // Child overrides with same type
            provide_context(AppConfig {
                name: "Child".to_string(),
                version: 2,
            });

            // Should get child's version
            let config = consume_context::<AppConfig>().unwrap();
            assert_eq!(config.name, "Child");
            assert_eq!(config.version, 2);
        });

        // After child scope, parent's context is still available
        let config = consume_context::<AppConfig>().unwrap();
        assert_eq!(config.name, "Parent");
    });
}

// ============================================================================
// 多类型 Context 测试
// ============================================================================

#[test]
fn test_multiple_context_types() {
    create_scope(|| {
        // Provide multiple different types
        provide_context(AppConfig {
            name: "App".to_string(),
            version: 1,
        });

        provide_context(Theme {
            primary_color: "blue".to_string(),
            font_size: 14,
        });

        provide_context(UserSession {
            user_id: 12345,
            token: "secret".to_string(),
        });

        // All should be retrievable
        assert!(consume_context::<AppConfig>().is_some());
        assert!(consume_context::<Theme>().is_some());
        assert!(consume_context::<UserSession>().is_some());

        // Check specific values
        let config = consume_context::<AppConfig>().unwrap();
        assert_eq!(config.name, "App");

        let theme = consume_context::<Theme>().unwrap();
        assert_eq!(theme.primary_color, "blue");

        let session = consume_context::<UserSession>().unwrap();
        assert_eq!(session.user_id, 12345);
    });
}

// ============================================================================
// Context 与 Effect 集成测试
// ============================================================================

#[test]
fn test_context_in_effect() {
    create_scope(|| {
        provide_context(AppConfig {
            name: "EffectTest".to_string(),
            version: 1,
        });

        let signal = Signal::new(0);
        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();

        create_effect(move || {
            let _val = *signal.read();
            
            // Access context inside effect
            if let Some(config) = consume_context::<AppConfig>() {
                *result_clone.borrow_mut() = config.name.clone();
            }
        });

        assert_eq!(*result.borrow(), "EffectTest");
    });
}

#[test]
fn test_context_across_effect_scopes() {
    create_scope(|| {
        provide_context(Theme {
            primary_color: "green".to_string(),
            font_size: 16,
        });

        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();

        create_effect(move || {
            // Access context directly in effect
            if let Some(theme) = consume_context::<Theme>() {
                *result_clone.borrow_mut() = theme.primary_color.clone();
            }
        });

        assert_eq!(*result.borrow(), "green");
    });
}

// ============================================================================
// 深层嵌套 Context 测试
// ============================================================================

#[test]
fn test_deep_context_lookup() {
    create_scope(|| {
        provide_context(AppConfig {
            name: "Root".to_string(),
            version: 1,
        });

        create_scope(|| {
            create_scope(|| {
                create_scope(|| {
                    create_scope(|| {
                        // 5 levels deep - should still find root context
                        let config = consume_context::<AppConfig>();
                        assert!(config.is_some());
                        assert_eq!(config.unwrap().name, "Root");
                    });
                });
            });
        });
    });
}

#[test]
fn test_context_at_each_level() {
    create_scope(|| {
        provide_context(AppConfig {
            name: "L1".to_string(),
            version: 1,
        });

        create_scope(|| {
            provide_context(AppConfig {
                name: "L2".to_string(),
                version: 2,
            });

            create_scope(|| {
                provide_context(AppConfig {
                    name: "L3".to_string(),
                    version: 3,
                });

                // Should get the closest one (L3)
                let config = consume_context::<AppConfig>();
                assert_eq!(config.as_ref().unwrap().name, "L3");
            });

            // Should get L2
            let config = consume_context::<AppConfig>();
            assert_eq!(config.as_ref().unwrap().name, "L2");
        });

        // Should get L1
        let config = consume_context::<AppConfig>();
        assert_eq!(config.as_ref().unwrap().name, "L1");
    });
}

// ============================================================================
// has_context 特定测试
// ============================================================================

#[test]
fn test_has_context_does_not_search_parents() {
    create_scope(|| {
        provide_context(AppConfig {
            name: "Parent".to_string(),
            version: 1,
        });

        // has_context in current scope
        assert!(has_context::<AppConfig>());

        create_scope(|| {
            // has_context only checks current scope, not parents
            // So this returns false even though parent has it
            assert!(!has_context::<AppConfig>());

            // But consume_context searches parents, so this works
            assert!(consume_context::<AppConfig>().is_some());
        });
    });
}

// ============================================================================
// Context 与 Cleanup 集成测试
// ============================================================================

#[test]
fn test_context_with_cleanup() {
    let cleanup_ran = Rc::new(RefCell::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    create_scope(move || {
        let config = AppConfig {
            name: "CleanupTest".to_string(),
            version: 1,
        };
        provide_context(config.clone());

        create_scope(move || {
             on_cleanup(move || {
                // Cleanup runs when inner scope is destroyed, but outer scope is still active
                if let Some(config) = consume_context::<AppConfig>() {
                    if config.name == "CleanupTest" {
                        *cleanup_ran_clone.borrow_mut() = true;
                    }
                }
            });
        });
    });

    assert!(*cleanup_ran.borrow());
}

// ============================================================================
// 实际使用场景测试
// ============================================================================

#[test]
fn test_theme_provider_pattern() {
    create_scope(|| {
        // Root provides theme
        provide_context(Theme {
            primary_color: "blue".to_string(),
            font_size: 16,
        });

        // Simulate component tree
        create_scope(|| {
            // Header component
            let theme = consume_context::<Theme>().unwrap();
            assert_eq!(theme.primary_color, "blue");

            create_scope(|| {
                // Button component inside header
                let theme = consume_context::<Theme>().unwrap();
                assert_eq!(theme.font_size, 16);
            });
        });

        create_scope(|| {
            // Footer component
            let theme = consume_context::<Theme>().unwrap();
            assert_eq!(theme.primary_color, "blue");
        });
    });
}

#[test]
fn test_dependency_injection_pattern() {
    #[derive(Clone)]
    struct Logger {
        prefix: String,
    }

    #[derive(Clone)]
    struct Database {
        connection: String,
    }

    create_scope(|| {
        // Provide dependencies
        provide_context(Logger {
            prefix: "[APP]".to_string(),
        });

        provide_context(Database {
            connection: "localhost:5432".to_string(),
        });

        // Service uses injected dependencies
        create_scope(|| {
            let logger = consume_context::<Logger>();
            let db = consume_context::<Database>();

            assert!(logger.is_some());
            assert!(db.is_some());
            assert_eq!(logger.unwrap().prefix, "[APP]");
            assert_eq!(db.unwrap().connection, "localhost:5432");
        });
    });
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_context_with_option_type() {
    #[derive(Clone, PartialEq, Debug)]
    struct OptionalConfig {
        value: Option<String>,
    }

    create_scope(|| {
        provide_context(OptionalConfig {
            value: Some("test".to_string()),
        });

        let config = consume_context::<OptionalConfig>();
        assert!(config.is_some());
        assert_eq!(config.unwrap().value, Some("test".to_string()));

        // Override with None
        provide_context(OptionalConfig { value: None });

        let config2 = consume_context::<OptionalConfig>();
        assert!(config2.is_some());
        assert_eq!(config2.unwrap().value, None);
    });
}

#[test]
fn test_context_with_vec_type() {
    #[derive(Clone, PartialEq)]
    struct Items {
        list: Vec<String>,
    }

    create_scope(|| {
        provide_context(Items {
            list: vec!["a".to_string(), "b".to_string()],
        });

        let items = consume_context::<Items>();
        assert_eq!(items.unwrap().list, vec!["a".to_string(), "b".to_string()]);
    });
}

#[test]
fn test_context_with_rc_type() {
    #[derive(Clone)]
    struct SharedState {
        data: Rc<RefCell<Vec<i32>>>,
    }

    create_scope(|| {
        let state = SharedState {
            data: Rc::new(RefCell::new(vec![1, 2, 3])),
        };

        let state_clone = state.clone();
        provide_context(state);

        let retrieved = consume_context::<SharedState>();
        assert!(retrieved.is_some());

        // Modify through one reference
        state_clone.data.borrow_mut().push(4);

        // Should see the change through the other
        let retrieved_unwrapped = retrieved.unwrap();
        let retrieved_data = retrieved_unwrapped.data.borrow();
        assert_eq!(*retrieved_data, vec![1, 2, 3, 4]);
    });
}
