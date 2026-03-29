/// Show a Windows toast notification using the Windows Runtime API.
/// Enhanced to support actions and sound effects.
fn show_windows_notification(
    title: &str,
    message: &str,
    actions: Option<Vec<(String, String)>>,
    sound: Option<String>,
) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::*;
    use windows::UI::Notifications::*;

    let toast_manager =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("NetSentry"))
            .map_err(|e| format!("Failed to create toast notifier: {:?}", e))?;

    let xml_doc = XmlDocument::new()
        .map_err(|e| format!("Failed to create XML document: {:?}", e))?;

    let mut toast_xml = format!(
        r#"<toast activationType="protocol" launch="netsentry://show">
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>"#,
        title, message
    );

    // Add actions if provided
    if let Some(actions) = actions {
        toast_xml.push_str("<actions>");
        for (label, arg) in actions {
            toast_xml.push_str(&format!(
                r#"<action content="{}" arguments="{}" activationType="protocol" />"#,
                label, arg
            ));
        }
        toast_xml.push_str("</actions>");
    }

    // Add sound if provided
    if let Some(sound) = sound {
        if sound == "default" {
            toast_xml.push_str(r#"<audio src="ms-winsoundevent:Notification.Default" />"#);
        } else if sound == "none" {
            toast_xml.push_str(r#"<audio silent="true" />"#);
        } else {
            // Custom sound, assume it's a path or event
            toast_xml.push_str(&format!(r#"<audio src="{}" />"#, sound));
        }
    }

    toast_xml.push_str("</toast>");

    xml_doc
        .LoadXml(&HSTRING::from(toast_xml))
        .map_err(|e| format!("Failed to load XML: {:?}", e))?;

    let toast = ToastNotification::CreateToastNotification(&xml_doc)
        .map_err(|e| format!("Failed to create toast: {:?}", e))?;

    toast_manager
        .Show(&toast)
        .map_err(|e| format!("Failed to show toast: {:?}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn show_native_notification(
    title: String,
    message: String,
    actions: Option<Vec<(String, String)>>,
    sound: Option<String>,
) -> Result<(), String> {
    show_windows_notification(&title, &message, actions, sound)
}
