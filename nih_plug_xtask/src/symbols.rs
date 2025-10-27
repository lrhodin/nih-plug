use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Check whether a binary exports the specified symbol. Used to detect the plugin formats supported
/// by a plugin library. Returns an error if the binary cuuld not be read. This function will also
/// parse non-native binaries.
pub fn exported<P: AsRef<Path>>(binary: P, symbol: &str) -> Result<bool> {
    // Parsing the raw binary instead of relying on nm-like tools makes cross compiling a bit easier
    let bytes = fs::read(&binary)
        .with_context(|| format!("Could not read '{}'", binary.as_ref().display()))?;
    match goblin::Object::parse(&bytes)? {
        goblin::Object::Elf(obj) => Ok(obj
            .dynsyms
            .iter()
            // We don't filter by functions here since we need to export a constant for CLAP
            .any(|sym| !sym.is_import() && obj.dynstrtab.get_at(sym.st_name) == Some(symbol))),
        goblin::Object::Mach(obj) => {
            let obj = match obj {
                goblin::mach::Mach::Fat(arches) => match arches
                    .get(0)
                    .context("Fat Mach-O binary without any binaries")?
                {
                    goblin::mach::SingleArch::MachO(obj) => obj,
                    // THis shouldn't be hit
                    goblin::mach::SingleArch::Archive(_) => {
                        anyhow::bail!(
                            "'{}' contained an unexpected Mach-O archive",
                            binary.as_ref().display()
                        )
                    }
                },
                goblin::mach::Mach::Binary(obj) => obj,
            };

            // XXX: Why are all exported symbols on macOS prefixed with an underscore?
            let symbol = format!("_{symbol}");

            // First check the dynamic symbol table (exports)
            if let Ok(exports) = obj.exports() {
                if exports.into_iter().any(|sym| sym.name == symbol) {
                    return Ok(true);
                }
            }

            // If not found in exports, check the regular symbol table
            // This is needed for cdylib builds which don't create a dynamic symbol table
            let symbols = obj.symbols();
            let mut found = false;
            let mut all_symbols = Vec::new();
            
            for sym_result in symbols {
                if let Ok((sym_name, nlist)) = sym_result {
                    all_symbols.push(sym_name);
                    // Check if this is a text symbol (function) with the right name
                    if sym_name == symbol && nlist.n_type & 0xe0 == 0x20 { // N_SECT | N_EXT
                        found = true;
                    }
                }
            }
            
            // If goblin didn't find the symbol, try using nm as a fallback
            if !found {
                if let Ok(output) = Command::new("nm")
                    .arg(binary.as_ref())
                    .output()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    found = output_str.contains(&symbol);
                }
            }
            
            Ok(found)
        }
        goblin::Object::PE(obj) => Ok(obj.exports.iter().any(|sym| sym.name == Some(symbol))),
        obj => bail!("Unsupported object type: {:?}", obj),
    }
}
