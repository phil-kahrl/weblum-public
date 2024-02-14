use leptos::*;
use crate::S3ObjectInfo;
use crate::ImageInfo;
use crate::ImageLink;

fn index_for_key(list: Vec<S3ObjectInfo>, key: String) -> usize {
    for (idx, el) in list.iter().enumerate() {
        if el.key() == key {
            return idx;
        }
    }
    0
}

#[component]
pub fn List(
    list: ReadSignal<Vec<S3ObjectInfo>>,
    current_image: WriteSignal<Option<String>>,
    read_current_image: ReadSignal<String>,
) -> impl IntoView {
    let list_size = list.get_untracked().len().try_into().expect("image list expected");
    let mut default_updates: Vec<RwSignal<bool>> = Vec::with_capacity(list_size);

    for _ in 0..list_size {
        default_updates.push(create_rw_signal(false));
    }
    let (updates_signal, _) = create_signal(default_updates);
    let default_current = create_rw_signal(false);

    let (local_current_image, set_local_current_image) = create_signal(None::<String>);

    create_effect(move |_| {
        for u in updates_signal.get_untracked() {
            if u.get_untracked() {
                u.set(false);
            }
        }
        let idx = index_for_key(list.get_untracked(), read_current_image.get());
        updates_signal.get_untracked().get(idx).unwrap_or(&default_current).set(true);
    });
    
    create_effect(move |_| {
        match local_current_image.get() {
            Some(i) => {
                // set the update signal.
                for u in updates_signal.get_untracked() {
                    if u.get_untracked() {
                        u.set(false);
                    }
                }
                current_image.set(Some(i.clone()));
                let idx = index_for_key(list.get_untracked(), i);
                updates_signal.get_untracked().get(idx).expect("signal expected").set(true);
            }
            None => (),
        }
    });

    view!{
        <div>
            <div>
                { move ||
                    list.get().to_vec().iter().into_iter()
                        .map(|contents| {
                            view!{
                                <div>
                                    <ImageLink
                                        contents={contents.clone()}
                                        set_current_image={set_local_current_image}
                                        current={*updates_signal.get().get(index_for_key(list.get_untracked(), contents.key())).unwrap_or(&default_current)}
                                    />
                                </div>

                            }
                        }).collect_view()
                }
            </div>
        </div>
    }
}
