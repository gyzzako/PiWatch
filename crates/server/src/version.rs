fn split_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut p = v.split('.');
    let major = p.next()?.parse().ok()?;
    let minor = p.next()?.parse().ok()?;
    let patch = p.next()?.parse().ok()?;
    Some((major, minor, patch))
}

pub(crate) fn check_compatible(agent_ver: &str, server_ver: &str) -> crate::error::Result<()> {
    let a = split_version(agent_ver).ok_or_else(|| {
        crate::error::Error::VersionMismatch(format!("Invalid agent version: {agent_ver}"))
    })?;
    let s = split_version(server_ver).ok_or_else(|| {
        crate::error::Error::Internal(format!("Invalid server version: {server_ver}"))
    })?;

    if a.0 < s.0 {
        return Err(crate::error::Error::VersionMismatch(format!(
            "Agent {agent_ver} is too old (server requires {server_ver})"
        )));
    }
    if a.0 == s.0 && a.1 < s.1 {
        return Err(crate::error::Error::VersionMismatch(format!(
            "Agent {agent_ver} is too old (server requires {server_ver})"
        )));
    }
    Ok(())
}
