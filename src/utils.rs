use directories::BaseDirs;
use std::path::PathBuf;

use crate::error::{MgdlError, MgdlResult};

pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => out.push('a'),
            'é' | 'è' | 'ê' | 'ë' => out.push('e'),
            'í' | 'ì' | 'î' | 'ï' => out.push('i'),
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => out.push('o'),
            'ú' | 'ù' | 'û' | 'ü' => out.push('u'),
            'ç' => out.push('c'),
            'ñ' => out.push('n'),
            'ý' | 'ÿ' => out.push('y'),
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => out.push('A'),
            'É' | 'È' | 'Ê' | 'Ë' => out.push('E'),
            'Í' | 'Ì' | 'Î' | 'Ï' => out.push('I'),
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => out.push('O'),
            'Ú' | 'Ù' | 'Û' | 'Ü' => out.push('U'),
            'Ç' => out.push('C'),
            'Ñ' => out.push('N'),
            'Ý' => out.push('Y'),
            c if c.is_ascii_alphanumeric() => out.push(c),
            _ => out.push('_'),
        }
    }

    // Collapse consecutive underscores and trim them from edges
    let mut result = String::with_capacity(out.len());
    let mut prev_underscore = true; // starts true to trim leading underscores
    for c in out.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    if result.ends_with('_') {
        result.pop();
    }

    result.to_lowercase()
}

pub fn extract_hash(url: &str) -> Option<String> {
    let path = url.trim_end_matches('/');
    let after_series = path.split("/series/").nth(1)?;
    let hash = after_series.split('/').next()?;
    if hash.is_empty() {
        return None;
    }
    Some(hash.to_string())
}

pub fn expand_tilde(path: PathBuf) -> MgdlResult<PathBuf> {
    if let Ok(stripped) = path.strip_prefix("~") {
        let base_dirs = BaseDirs::new()
            .ok_or_else(|| MgdlError::Config("Could not determine home directory".to_string()))?;
        return Ok(base_dirs.home_dir().join(stripped));
    }

    Ok(path)
}
