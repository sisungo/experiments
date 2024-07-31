use crate::{AccessVector, Decision};
use native_dialog::MessageType;

pub fn ask_for_permission(access_vector: &AccessVector) -> anyhow::Result<Decision> {
    // FIXME: Due to limitations of `native-dialog`, we cannot ask for `AllowOnce` or `DenyOnce` yet.
    // FIXME: Transform subject into display name
    let decision = native_dialog::MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title(&access_vector.subject_cell)
        .set_text(&format!("")[..])
        .show_confirm()?;
    Ok(match decision {
        true => Decision::Allow,
        false => Decision::Deny,
    })
}
