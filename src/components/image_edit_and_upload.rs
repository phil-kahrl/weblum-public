use leptos::*;
use leptonic::prelude::*;
use leptos_icons::FaIcon::*;
use leptos_icons::BiIcon::BiDownloadSolid;

use time::OffsetDateTime;
use time::Duration;

use std::io::Cursor;

use image::DynamicImage;
use image::io::Reader;
use image::imageops::FilterType;
use image::GenericImageView;

use js_sys::Uint8Array;

use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;

use web_sys::File;
use web_sys::HtmlInputElement;
use web_sys::EventTarget;

use gloo_timers::future::TimeoutFuture;

use rexif::parse_buffer;

use img_parts::jpeg::Jpeg;
use img_parts::{ImageEXIF};

use crate::parse_image_metadata;
use crate::update_image_metadata_from_binary;
use crate::upload_image_1;
use crate::get_device_type;
use crate::encode_binary;

use crate::EditSizeControl;
use crate::UploadFileControl;
use crate::EditedImageDisplay;
use crate::ImageMetaDataDisplay;
use crate::DeviceType;

#[derive(Clone)]
pub struct ImageMetadata {
    pub mime_type: String,
    pub filename: String,
    pub last_modified: OffsetDateTime,
    pub size: f64,
    pub dimensions: Option<(u32, u32)>,
    pub lat: Option<String>,
    pub long: Option<String>,
    pub raw_binary: Option<Vec<u8>>,
    pub all_tags: Vec<(String, String)>,
}

impl ImageMetadata {
    pub fn new(_file: File) -> Self {
        let unix_epoch = (_file.last_modified()/1000.0).round() as i64;
        let duration = Duration::new(unix_epoch, 0);
        let epoch_time = OffsetDateTime::UNIX_EPOCH;
        let date = epoch_time.checked_add(duration).expect("unable to create time");

        Self {
            filename: _file.name(),
            mime_type: _file.type_(),
            last_modified: date,
            size: _file.size(),
            dimensions: None,
            lat: None,
            long: None,
            raw_binary: None,
            all_tags: vec![].into(),
        }
    }

    pub fn set_raw_binary(&mut self, raw: Vec<u8>) {
        self.raw_binary = Some(raw)
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) {
        self.dimensions = Some(dimensions)
    }

    pub fn get_lat(self) -> String {
        match self.lat {
            Some(lat) => lat,
            None => "No latitude tag found.".to_string(),
        }
    }

    pub fn get_long(self) -> String {
        match self.long {
            Some(long) => long,
            None => "No longitude tag found.".to_string(),
        }
    }

    pub fn set_lat(&mut self, l: String) {
        self.lat = Some(l);
    }

    pub fn set_long(&mut self, l: String) {
        self.long = Some(l);
    }

    pub fn _to_string(&self) -> String {
        let result = format!("mime: {}", self.mime_type);
        let result1 = match &self.lat {
            Some(l) => format!("{}, lat: {}", result, l),
            None => result,
        };
        match &self.long {
            Some(l) => format!("{}, long: {}", result1, l),
            None => result1,
        }
    }
}


#[derive(Clone, PartialEq)]
pub enum EditState {
    New,
    Initializing,
    Updating,
    Ready,
}

#[derive(Clone)]
pub struct SizeInputs {
    dimensions: (u32, u32),
    quality: u8,
}

impl SizeInputs {
    pub fn new(dimensions: (u32, u32), quality: u8) -> Self {
        Self {
            dimensions,
            quality,
        }
    }
}

#[derive(Clone)]
pub struct SizeAttributes {
    dimensions: (u32, u32),
    quality: u8,
    file_size: usize,
}

impl SizeAttributes {
    pub fn new(dimensions: (u32, u32), file_size: usize, quality: u8) -> Self {
        Self {
            dimensions,
            file_size,
            quality,
        }
    }

    pub fn _to_string(self) -> String {
        format!(" dimensions {:?}, file size: {}, quality: {}", self.dimensions, self.file_size, self.quality)
    }
}

#[derive(Clone)]
pub enum UploadStatus {
    Idle,
    InProgress,
    Success,
    Failure,
}

#[component]
pub fn ImageEditor(
    default_filename: String,
    default_size_attributes: SizeAttributes,
    image_metadata: ImageMetadata,
    set_image_metadata: WriteSignal<Option<ImageMetadata>>,
    read_file_binary: ReadSignal<Vec<u8>>,
    set_file_binary: WriteSignal<Vec<u8>>,
    upload_status: ReadSignal<UploadStatus>,
) -> impl IntoView {
    // SIGNALS

    let (read_image_metadata, _) = create_signal(image_metadata);
    // dynamic engine for editing operations
    let dynamic_image_default: Option<DynamicImage> = None;
    let (dynamic_image, set_dynamic_image) = create_signal(dynamic_image_default);
    
    // the name of the file
    let (filename, _) = create_signal(default_filename);

    // size attributes
    let (size_attributes, set_size_attributes) = create_signal(Some(default_size_attributes.clone()));

    // size inputs from the user
    let (size_inputs, set_set_size_inputs) = create_signal(SizeInputs::new(
        default_size_attributes.clone().dimensions,
        default_size_attributes.clone().quality,
    ));

    // enumerated state of the editor.
    let (edit_state, set_edit_state) = create_signal(EditState::New);

    // exif tags removed from image
    let (exif_removed, set_exif_removed) = create_signal(false);

    // ACTIONS
    // creates the dynamic image from the binary and updates the dynamic image signal
    let initialize = create_action(move |_: &String| async move {
        set_edit_state.set(EditState::Initializing);
        TimeoutFuture::new(1).await;

        let dimensions = Reader::new(Cursor::new(read_file_binary.get_untracked()))
            .with_guessed_format()
            .expect("Cursor io never fails")
            .into_dimensions();

        match dimensions {
            Ok(dims) => {
                let size = read_file_binary.get_untracked().len();
                let sa = SizeAttributes::new(dims, size, size_inputs.get_untracked().quality);
                set_size_attributes.set(Some(sa));
            },
            Err(_) => log::info!("No dimensions from image binary"),
        }

        let reader = Reader::new(Cursor::new(read_file_binary.get_untracked()))
            .with_guessed_format()
            .expect("Cursor io never fails");


        set_edit_state.set(EditState::Updating);
        TimeoutFuture::new(1).await;
        
        match reader.decode() {
            Ok(di) => {
                set_dynamic_image.set(Some(di));
            },
            Err(_) => log::info!("Error decoding image"),
        }
        set_edit_state.set(EditState::Ready);
    });

    let remove_exif = create_action(move |_: &String| async move {
        set_edit_state.set(EditState::Updating);
        TimeoutFuture::new(1).await;
        log::info!("Remove Exif:");
        let jpeg = Jpeg::from_bytes(read_file_binary.get_untracked().into());
        let empty: Vec<u8> = vec![];
        match jpeg {
            Ok(mut jp) => {
                log::info!("Removing Exif tags");
                jp.set_exif(Some(empty.into()));
                let encoder = jp.encoder();
                let out: Vec<u8> = vec![];
                let _ = encoder.clone().write_to(out);
                set_file_binary.set(encoder.bytes().into());
                log::info!("Done removing Exif tags");
                initialize.dispatch("".to_string());
                set_edit_state.set(EditState::Ready);

                // verify exif removed
                match parse_buffer(&read_file_binary.get_untracked()) {
                    Ok(result) => {    
                        for entry in result.entries {
                            if entry.tag.to_string() == "GPS latitude" {
                                log::error!("lat found")
                            }
                            if entry.tag.to_string() == "GPS longitude" {
                                log::error!("long found")
                            }
                        }
                    },
                    Err(e) => log::info!("parse buffer failed {}", e),
                }
            },
            Err(_) => log::info!("Jpeg not decoded."),
        }
        set_exif_removed.set(true);
    });

    // parse the binary from the DynamicImage and update the file binary signal
    let decode_from_dynamic_image = create_action(move |_: &String| async move {
        set_edit_state.set(EditState::Updating);
        TimeoutFuture::new(1).await;
        let image = dynamic_image.get_untracked().expect("image expected");
        let buffer: Vec<u8> = vec![];
        let mut buf_writer = Cursor::new(buffer);

        match image.write_to(&mut buf_writer, image::ImageOutputFormat::Jpeg(size_inputs.get_untracked().quality as u8)) {
            Ok(_) => (),
            Err(_) => log::info!("buffer write failed"),
        }
        set_file_binary.set(buf_writer.get_ref().to_vec());

        let updated_image_metadata = update_image_metadata_from_binary(read_image_metadata.get_untracked(), read_file_binary.get_untracked());
        set_image_metadata.set(updated_image_metadata);
        set_edit_state.set(EditState::Ready);
    });

    // rotates the dynamic image by 90 degrees and updates the dynamic image signal
    let rotate_image =  create_action(move |_: &String| async move {
        set_edit_state.set(EditState::Updating);
        TimeoutFuture::new(1).await;
        let image = dynamic_image.get_untracked().expect("image expected");
        set_dynamic_image.set(Some(image.rotate90()));
        decode_from_dynamic_image.dispatch("rotate_action".to_string());
    });

    // resize the image to the size specified by the size attributes signal and write to the dynamic image signal.
    let resize_image = create_action(move |_: &String| async move {
        TimeoutFuture::new(1).await;
        set_edit_state.set(EditState::Updating);
        TimeoutFuture::new(1).await;
        let new_width = size_inputs.get_untracked().dimensions.0;
        match dynamic_image.get_untracked() {
            Some(image) => {
                set_dynamic_image.set(Some(image.resize(new_width, new_width, FilterType::Nearest)));
                decode_from_dynamic_image.dispatch("resize_action".to_string());
            },
            None => (),
        }
    });

    // upload event
    let (_start_upload, _set_start_upload) = create_signal(false);

    // listener for the dynamic image change.
    create_effect(move |_| {
        match dynamic_image.get() {
            Some(di) => {
                let new_dims = di.dimensions();
                match size_attributes.get_untracked() {
                    Some(sa) => {
                        set_size_attributes.set(Some(SizeAttributes::new(new_dims, sa.file_size, sa.quality)));
                    },
                    None => {
                        set_size_attributes.set(Some(SizeAttributes::new(new_dims, 0, 100)));
                    },
                }
            },
            None => ()
        }
    });

    //listener for resize input changes
    create_effect(move |_| {
        let new_size = size_inputs.get();
        let current_size = size_attributes.get_untracked().expect("size attributes");
        if new_size.dimensions.0 == current_size.dimensions.0 {
            //
        } else {
            resize_image.dispatch("resize listener".to_string());
        }
    });

    // listener for the file binary change.
    create_effect(move |_| {
        let fb = read_file_binary.get();
        let current = size_attributes.get_untracked().expect("");
        set_size_attributes.set(Some(SizeAttributes::new(current.dimensions, fb.len(), current.quality)));
    });

    initialize.dispatch("image editor".to_string());

    view! {
        <div style="display: flex; flex-direction: column; width: 100%;">
            <div style="display: flex; flex-direction: column;">                
                <div style="display: flex; flex-direction: row; justify-content: space-between; width: 360px;">
                    <EditSizeControl
                        disabled = Signal::derive(move || { edit_state.get() != EditState::Ready} )
                        resize_listener={set_set_size_inputs}
                    />
                    <div title="Rotate Image">
                        <Button
                            disabled = Signal::derive(move || { edit_state.get() != EditState::Ready} )
                            on_click=move |_ev| {
                                rotate_image.dispatch("".to_string());
                            }>
                                <div>
                                    <Icon icon=leptos_icons::Icon::from(FaRotateRightSolid) />
                                    <div style="font-size: .5em;">"Rotate"</div>
                                </div>
                        </Button>
                    </div>
                    <div title={move || if exif_removed.get() {"No Exif Tags Exist"} else {"De-Tag: Remove Exif Tags"}}>
                        <Button
                            disabled = Signal::derive(move || { edit_state.get() != EditState::Ready || exif_removed.get()} )
                            on_click=move |_ev| {
                                remove_exif.dispatch("".to_string());
                            }
                        >
                            <div>
                                <Icon icon=leptos_icons::Icon::from(FaEraserSolid) />
                                <div style="font-size: .5em;">"De-Tag"</div>
                            </div>
                        </Button>
                    </div>

                    <div style="width: 100px; max-width: 63px; font-size: 0.8em; overflow: hidden;">
                        {move ||
                            match upload_status.get() {
                                UploadStatus::Idle => "No uploads.".to_string(),
                                UploadStatus::InProgress =>  format!("uploading: {}", filename.get()),
                                UploadStatus::Success => format!("uploaded: {}", filename.get()),
                                UploadStatus::Failure => format!("upload failed: {}", filename.get()),
                            }
                        }
                    </div>

                    </div>
                    <div style="display:flex; flex-direction: row; font-size: .8em;">
                        <div>
                            {move || 
                                match size_attributes.get() {
                                    Some(sa) => {
                                        let savings = (sa.file_size as f64/read_image_metadata.get_untracked().size as f64)*100.0;
                                        let dims = format!("{} x {} ", sa.dimensions.0, sa.dimensions.1);
                                        let size = format!("{:.3} Mb", sa.file_size as f64/1000000.0);
                                        view!{<div style="display:flex; flex-direction: row;">
                                            <div style="padding: 5px;">{dims}</div>
                                            <div style="padding: 5px;">{size} {format!(" ({:.2}%)", savings)}</div>
                                        </div>
                                    }},
                                    None => {
                                        view!{<div>"None"</div>}
                                    },
                                }
                            }
                        </div>
                    

                        <div style="width: 80px; margin: 5px; font-size: .8em;">
                        {move || match edit_state.get() {
                            EditState::New => {
                                view!{
                                    <div>
                                        <Chip color=ChipColor::Primary>"New"</Chip>
                                    </div>
                                }   
                            },
                            EditState::Initializing => {
                                view!{
                                <div>
                                    <Chip color=ChipColor::Secondary>"Initializing"</Chip>
                                    </div>
                                }   
                            },
                            EditState::Updating => {
                                view!{
                                    <div>
                                        <Chip color=ChipColor::Secondary>"Updating"</Chip>
                                    </div>
                                }   
                            },
                            EditState::Ready => {
                                view!{
                                    <div>
                                        <Chip color=ChipColor::Success>"Ready"</Chip>
                                    </div>
                                }   
                            },
                        }}
                    </div>

                { move || {
                    view!{
                        <a
                            download={filename.get()}
                            href={format!("data:image/jpeg;base64,{}", encode_binary(read_file_binary.get()))}
                        >
                            <span title="Download image" style="font-size: 1.5em;">
                                <Icon icon=leptos_icons::Icon::from(BiDownloadSolid) />
                            </span>
                        </a>
                        }.into_view()
                    }}
                </div>

            </div>
            <EditedImageDisplay
                file_binary = {read_file_binary}
            />
        </div>
    }
}

#[component]
pub fn FileEditAndPublishControl (
    update_list: WriteSignal<bool>,
    set_show_modal: WriteSignal<bool>,
) -> impl IntoView {

    // signal for image data derived from the user selected File object.
    let image_metadata_option: Option<ImageMetadata> = None;
    let (image_metadata, set_image_metadata) = create_signal(image_metadata_option);

    let edited_image_metadata_option: Option<ImageMetadata> = None;
    let (edited_image_metadata, set_edited_image_metadata) = create_signal(edited_image_metadata_option);

    // edited image dimensions, file size and quality.
    let image_info_default = SizeAttributes::new((1200,1200), 0, 50);
    let (_image_info, _set_image_info) = create_signal(image_info_default);

    let file_option: Option<File> = None;
    let (file, set_file) = create_signal(file_option);
    let file_binary_default: Vec<u8> = vec![];

    let (read_file_binary, set_file_binary) = create_signal(file_binary_default);
    let (default_upload_filename, set_default_upload_filename) = create_signal( "filename".to_string());

    let (disabled, set_disabled) = create_signal(true);
    let update_image_data = create_action(move |_: &String| async move {
        let file = file.get_untracked().expect("file expected");
        let parsed = parse_image_metadata(file).await.expect("parsed");
        set_image_metadata.set(Some(parsed.clone()));
        set_default_upload_filename.set(parsed.clone().filename);
        set_file_binary.set(parsed.clone().raw_binary.expect(""));
        set_edited_image_metadata.set(Some(parsed));
    });

    create_effect(move |_| {
        match file.get() {
            Some(f) => {
                set_default_upload_filename.set(f.name());
                set_disabled.set(false);
            },
            None => (),
        }
    });

    create_effect(move |_| {
        match image_metadata.get() {
            Some(im) => {
                set_default_upload_filename.set(im.clone().filename);
                set_file_binary.set(im.clone().raw_binary.expect(""));
            },
            None => (),
        }
    });

    let (upload_status, set_upload_status) = create_signal(UploadStatus::Idle);
    // upload event
    let (start_upload, set_start_upload) = create_signal(None::<String>);

    let upload_image = create_action(move |_: &String| async move {
        set_upload_status.set(UploadStatus::InProgress);
        TimeoutFuture::new(1).await;
        let fb = read_file_binary.get_untracked();
        let u = Uint8Array::new_with_length(fb.len() as u32);
        u.copy_from(&fb);

        let image_name = match start_upload.get_untracked() {
            Some(n) => n,
            None => default_upload_filename.get_untracked()
        };

        match upload_image_1(u.into(), format!("images/{}", image_name)).await {
            Ok(_) => {
                set_upload_status.set(UploadStatus::Success);
                update_list.set(true);
            },
            Err(_) => {
                log::info!("Upload failed");
                set_upload_status.set(UploadStatus::Failure);
            }
        }
        set_start_upload.set(None);
    });

    //listener for upload event
    create_effect(move |_| {
        match start_upload.get() {
            Some(_) => {
                upload_image.dispatch("upload".to_string());
            },
            None => (),
        }
    });

    view!{ 
        <div>
            <div id="fileSelectControl"
                style = {match get_device_type() {
                    DeviceType::Mobile => "display:flex; flex-direction: column; margin: 0 0 1em 0; width: 310px;",
                    DeviceType::Desktop => "display:flex; flex-direction: column; margin: 0 0 1em 0;",
                }}
            >
                <input 
                    type="file" 
                    id="fileUpload"
                    on:change=move |ev| {
                        let t = ev.target();
                        let et: EventTarget = t.expect("target");
                        let r: &JsValue = et.as_ref();
                        let file_input = r.clone().dyn_into::<HtmlInputElement>();
                        let files = file_input.expect("file input").files();
                        let file = files.expect("files").item(0).expect("file");
                        set_file.set(Some(file.clone()));
                        set_default_upload_filename.set(file.name());
                        update_image_data.dispatch("".to_string());         
                    }
                />
                
                <div style="padding: 10px 0 0 0; display: flex; flex-direction: row; font-size: .8em;">
                    <UploadFileControl
                        disabled={disabled}
                        start_upload={set_start_upload}
                        default_upload_filename={default_upload_filename}
                    />
                    <Button
                    on_click=move |_| {
                        set_show_modal.set(false);
                    }
                >
                    "Close"
                </Button>
                </div>
            </div>
            <Tabs mount=Mount::Once>
                <Tab name="tab-1" label="Edit".into_view() >
                    <div>
                        {move || match image_metadata.get() {
                            Some(im) => view!{
                                <div>
                                    <ImageEditor
                                        image_metadata={im.clone()}
                                        set_image_metadata={set_edited_image_metadata}
                                        read_file_binary={read_file_binary}
                                        set_file_binary={set_file_binary}
                                        default_filename={im.filename}
                                        default_size_attributes={
                                            SizeAttributes::new(im.dimensions.expect("dims"), im.size.floor() as usize, 100)
                                        }
                                        upload_status={upload_status}
                                    />
                                </div>},
                            None => view!{
                                <div>"No Image File Selected"</div>
                            },
                        }}
                    </div>
                </Tab>
                <Tab name="tab-2" label="Image Attributes".into_view() >
                    <div style="display: flex; flex-direction: row; width: 100%; height: 100%;">
                        <ImageMetaDataDisplay 
                            image_metadata={image_metadata}
                            edited_image_metadata={edited_image_metadata} 
                        />
                    </div>
                </Tab>
            </Tabs>
        </div>
    }
}