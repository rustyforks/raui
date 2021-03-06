#![cfg(test)]

use crate::prelude::*;
use std::str::FromStr;

#[test]
fn test_macro() {
    fn app(_context: WidgetContext) -> WidgetNode {
        widget! {()}
    }

    fn text(_context: WidgetContext) -> WidgetNode {
        widget! {()}
    }

    println!("{:#?}", widget! {()});
    println!(
        "{:#?}",
        widget! {
            (app)
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()})
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app {
                ass = (text: {"ass".to_owned()})
                hole = ()
            })
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()} {
                ass = (text: {"ass".to_owned()})
                hole = ()
            })
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (app: {()} [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (#{"app"} app {
                ass = (text: {"ass".to_owned()})
                hole = ()
            } [
                (text: {"hole".to_owned()})
                ()
            ])
        }
    );
    println!(
        "{:#?}",
        widget! {
            (#{42} app: {()} {
                ass = (text: {"ass".to_owned()})
                hole = {widget! {()}}
            } [
                (text: {"hole".to_owned()})
                {{WidgetUnit::None}}
                {{WidgetNode::None}}
            ])
        }
    );
}

#[test]
#[allow(dead_code)]
#[cfg(feature = "html")]
fn test_hello_world() {
    use std::convert::TryInto;

    #[derive(Debug, Default, Copy, Clone)]
    struct AppProps {
        pub index: usize,
    }
    implement_props_data!(AppProps);

    // convenient macro that produces widget component processing function.
    widget_component! {
        // <component name> ( [list of context data to unpack into scope] )
        app(props, named_slots) {
            // easy way to get widgets from named slots.
            unpack_named_slots!(named_slots => { title, content });
            let index = props.read::<AppProps>().map(|p| p.index).unwrap_or(0);

            // we always return new widgets tree.
            widget! {
                // Forgive me the syntax, i'll make a JSX-like one soon using procedural macros.
                // `#{key}` - provided value gives a unique name to node. keys allows widgets
                //      to save state between render calls. here we just pass key of this widget.
                // `vertical_box` - name of widget component to use.
                // `[...]` - listed widget slots. here we just put previously unpacked named slots.
                (#{index} vertical_box [
                    {title}
                    {content}
                ])
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone)]
    struct ButtonState {
        pub pressed: bool,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum ButtonAction {
        Pressed,
        Released,
    }

    widget_hook! {
        use_empty(life_cycle) {
            life_cycle.mount(|_, _, _, _, _| {
                println!("=== BUTTON MOUNTED");
            });

            life_cycle.change(|_, _, _, _, _| {
                println!("=== BUTTON CHANGED");
            });

            life_cycle.unmount(|_, _, _, _| {
                println!("=== EMPTY UNMOUNTED");
            });
        }
    }

    // you use life cycle hooks for storing closures that will be called when widget will be
    // mounted/changed/unmounted. they exists for you to be able to resuse some common logic across
    // multiple components. each closure provides arguments such as:
    // - widget id
    // - widget state
    // - message sender (this one is used to message other widgets you know about)
    // - signal sender (this one is used to message application host)
    // although this hook uses only life cycle, you can make different hooks that use many
    // arguments, even use context you got from the component!
    widget_hook! {
        use_button(key, life_cycle) [use_empty] {
            let key_ = key.to_owned();
            life_cycle.mount(move |_, _, state, _, _| {
                println!("=== BUTTON MOUNTED: {}", key_);
                drop(state.write(ButtonState { pressed: false }));
            });

            let key_ = key.to_owned();
            life_cycle.change(move |_, _, state, messenger, signals| {
                println!("=== BUTTON CHANGED: {}", key_);
                for msg in messenger.messages {
                    if let Some(msg) = msg.downcast_ref::<ButtonAction>() {
                        let pressed = match msg {
                            ButtonAction::Pressed => true,
                            ButtonAction::Released => false,
                        };
                        println!("=== BUTTON ACTION: {:?}", msg);
                        drop(state.write(ButtonState { pressed }));
                        drop(signals.write(Box::new(*msg)));
                    }
                }
            });

            let key_ = key.to_owned();
            life_cycle.unmount(move |_, _, _, _| {
                println!("=== BUTTON UNMOUNTED: {}", key_);
            });
        }
    }

    widget_component! {
        button(key, props) [use_button] {
            println!("=== PROCESS BUTTON: {}", key);

            widget!{
                (#{key} text: {props})
            }
        }
    }

    widget_component! {
        title_bar(key, props) {
            let title = props.read_cloned_or_default::<String>();

            widget! {
                (#{key} text: {title})
            }
        }
    }

    widget_component! {
        vertical_box(id, key, listed_slots) {
            // listed slots are just widget node children.
            // here we just unwrap widget units (final atomic UI elements that renderers read).
            let items = listed_slots
                .into_iter()
                .map(|slot| FlexBoxItem {
                    slot: slot.try_into().expect("Cannot convert slot to WidgetUnit!"),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            // we use `{{{ ... }}}` to inform macro that this is widget unit.
            widget! {{{
                FlexBox {
                    id: id.to_owned(),
                    items,
                    ..Default::default()
                }
            }}}
        }
    }

    widget_component! {
        text(id, key, props) {
            let text = props.read_cloned_or_default::<String>();

            widget!{{{
                TextBox {
                    id: id.to_owned(),
                    text,
                    ..Default::default()
                }
            }}}
        }
    }

    let mut application = Application::new();
    let tree = widget! {
        (app {
            // <named slot name> = ( <widget to put in a slot> )
            title = (title_bar: {"Hello".to_owned()})
            content = (vertical_box [
                (#{"hi"} button: {"Say hi!".to_owned()})
                (#{"exit"} button: {"Close".to_owned()})
            ])
        })
    };
    println!("=== INPUT:\n{:#?}", tree);

    // some dummy widget tree renderer.
    // it reads widget unit tree and transforms it into target format.
    let mut renderer = HtmlRenderer::default();

    println!("=== PROCESS");
    // `apply()` sets new widget tree.
    application.apply(tree);
    // `render()` calls renderer to perform transformations on processed application widget tree.
    if let Ok(output) = application.render(&mut renderer) {
        println!("=== OUTPUT:\n{}", output);
    }

    println!("=== PROCESS");
    // by default application won't process widget tree if nothing was changed.
    // "change" is either any widget state change, or new message sent to any widget (messages
    // can be sent from application host, for example a mouse click, or from another widget).
    application.forced_process();
    if let Ok(output) = application.render(&mut renderer) {
        println!("=== OUTPUT:\n{}", output);
    }

    let tree = widget! {
        (app)
    };
    println!("=== INPUT:\n{:#?}", tree);
    println!("=== PROCESS");
    application.apply(tree);
    if let Ok(output) = application.render(&mut HtmlRenderer::default()) {
        println!("=== OUTPUT:\n{}", output);
    }
}

#[test]
fn test_layout_no_wrap() {
    let mut layout_engine = DefaultLayoutEngine::default();
    let view = Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    };

    let tree = widget! {{{
        FlexBox {
            id: WidgetId::from_str("type:/list").unwrap(),
            direction: FlexBoxDirection::VerticalTopToBottom,
            separation: 10.0,
            items: vec![
                FlexBoxItem {
                    fill: 1.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/0").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(100.0),
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
                FlexBoxItem {
                    fill: 1.0,
                    grow: 1.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/1").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Fill,
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
                FlexBoxItem {
                    fill: 1.0,
                    grow: 2.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/2").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Fill,
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }}};

    let mut application = Application::new();
    application.apply(tree);
    application.forced_process();
    println!(
        "=== TREE INSPECTION:\n{:#?}",
        application.rendered_tree().inspect()
    );
    if application.layout(view, &mut layout_engine).is_ok() {
        println!("=== LAYOUT:\n{:#?}", application.layout_data());
    }
}

#[test]
fn test_layout_wrapping() {
    let mut layout_engine = DefaultLayoutEngine::default();
    let view = Rect {
        left: 0.0,
        right: 1024.0,
        top: 0.0,
        bottom: 576.0,
    };

    let tree = widget! {{{
        FlexBox {
            id: WidgetId::from_str("type:/list").unwrap(),
            direction: FlexBoxDirection::HorizontalLeftToRight,
            separation: 10.0,
            wrap: true,
            items: vec![
                FlexBoxItem {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 1.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/0").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(100.0),
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
                FlexBoxItem {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 1.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/1").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(200.0),
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
                FlexBoxItem {
                    basis: Some(400.0),
                    fill: 1.0,
                    grow: 2.0,
                    slot: SizeBox {
                        id: WidgetId::from_str("type:/list/2").unwrap(),
                        width: SizeBoxSizeValue::Fill,
                        height: SizeBoxSizeValue::Exact(50.0),
                        ..Default::default()
                    }.into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }}};

    let mut application = Application::new();
    application.apply(tree);
    application.forced_process();
    println!(
        "=== TREE INSPECTION:\n{:#?}",
        application.rendered_tree().inspect()
    );
    if application.layout(view, &mut layout_engine).is_ok() {
        println!("=== LAYOUT:\n{:#?}", application.layout_data());
    }
}
