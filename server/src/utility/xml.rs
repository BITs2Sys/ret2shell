//! XML parsers
//! 
//! This module contains parsers for XML responses from other old servers.

use serde::{Deserialize, Serialize};

/// Student info in yale.edu schema.
#[derive(Serialize, Deserialize)]
pub struct IdsInfo {
    /// Student name
    pub name: String,
    /// Student ID
    pub uid: String,
}

#[allow(dead_code)]
/// Parse the XML response from `yale.edu` schema and get the student info.
pub fn get_student_info_from_xml_yale_edu(xml_response: impl AsRef<str>) -> anyhow::Result<IdsInfo> {
    let doc = roxmltree::Document::parse(xml_response.as_ref())?;
    let name_node = doc
        .descendants()
        .find(|node| node.tag_name().name() == "cn")
        .ok_or(anyhow::anyhow!("auth xml has no name attribute"))?;
    let uid_node = doc
        .descendants()
        .find(|node| node.tag_name().name() == "uid")
        .ok_or(anyhow::anyhow!("auth xml has no uid attribute"))?;
    let name = name_node
        .text()
        .ok_or(anyhow::anyhow!("name node has no text"))?
        .to_owned();
    let uid = uid_node
        .text()
        .ok_or(anyhow::anyhow!("uid node has no text"))?
        .to_owned();
    Ok(IdsInfo { name, uid })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_parse() {
        let dx_xml = r#"
<cas:serviceResponse xmlns:cas='http://www.yale.edu/tp/cas'>
    <cas:authenticationSuccess>
        <cas:user>1145141919810</cas:user>
        <cas:attributes>
            <cas:isFromNewLogin>true</cas:isFromNewLogin>
            <cas:authenticationDate>2023-04-29T09:29:17.370+08:00[GMT+08:00]</cas:authenticationDate>
            <cas:loginType>1</cas:loginType>
            <cas:successfulAuthenticationHandlers>com.wisedu.minos.config.login.RememberMeUsernamePasswordHandler</cas:successfulAuthenticationHandlers>
            <cas:cn>田所浩二</cas:cn>
            <cas:bindUserList>1145141919810</cas:bindUserList>
            <cas:userName>田所浩二</cas:userName>
            <cas:samlAuthenticationStatementAuthMethod>urn:oasis:names:tc:SAML:1.0:am:unspecified</cas:samlAuthenticationStatementAuthMethod>
            <cas:credentialType>MyRememberMeCaptchaCredential</cas:credentialType>
            <cas:uid>1145141919810</cas:uid>
            <cas:authenticationMethod>com.wisedu.minos.config.login.RememberMeUsernamePasswordHandler</cas:authenticationMethod>
            <cas:longTermAuthenticationRequestTokenUsed>false</cas:longTermAuthenticationRequestTokenUsed>
            <cas:cllt>userNameLogin</cas:cllt>
            <cas:dllt>generalLogin</cas:dllt>
            </cas:attributes>
    </cas:authenticationSuccess>
</cas:serviceResponse>
        "#;
        let info = get_student_info_from_xml_yale_edu(dx_xml).unwrap();
        assert_eq!(info.name, "田所浩二");
        assert_eq!(info.uid, "1145141919810");
    }
}
