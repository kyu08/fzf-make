// module for file manipulation
use std::path::Path;

use crate::parser::{self, makefile};

// get_makefile_file_names returns filenames of Makefile and the files included by Makefile
pub fn create_makefile() -> Result<makefile::Makefile, &'static str> {
    let Some(makefile_name) = specify_makefile_name(".".to_string()) else { return Err("makefile not found") };

    Ok(parser::makefile::Makefile::new(
        Path::new(&makefile_name).to_path_buf(),
    )?)
}

fn specify_makefile_name(target_path: String) -> Option<String> {
    //  By default, when make looks for the makefile, it tries the following names, in order: GNUmakefile, makefile and Makefile.
    //  https://www.gnu.org/software/make/manual/make.html#Makefile-Names
    // enumerate `Makefile` too not only `makefile` to make it work on case insensitive file system
    let makefile_name_options = vec!["GNUmakefile", "makefile", "Makefile"];

    for file_name in makefile_name_options {
        let path = Path::new(&target_path).join(file_name);
        if path.is_file() {
            return Some(file_name.to_string());
        }
        continue;
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{self, File};
    use uuid::Uuid;

    #[test]
    fn specify_makefile_name_test() {
        struct Case {
            title: &'static str,
            files: Vec<&'static str>,
            expect: Option<String>,
        }
        let cases = vec![
            Case {
                title: "no makefile",
                files: vec!["README.md"],
                expect: None,
            },
            Case {
                title: "GNUmakefile",
                files: vec!["makefile", "GNUmakefile", "README.md", "Makefile"],
                expect: Some("GNUmakefile".to_string()),
            },
            Case {
                title: "makefile",
                files: vec!["makefile", "Makefile", "README.md"],
                expect: Some("makefile".to_string()),
            },
            // NOTE: not use this test case because there is a difference in handling case sensitivity between macOS and linux.
            // to use this test case, you need to determine whether the file system is
            // case-sensitive or not when test execute.
            // Case {
            // title: "Makefile",
            // files: vec!["Makefile", "README.md"],
            // expect: Some("makefile".to_string()),
            // },
        ];

        for case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            match fs::create_dir(tmp_dir.as_path()) {
                Err(e) => panic!("fail to create dir: {:?}", e),
                Ok(_) => {}
            }

            for file in case.files {
                match File::create(tmp_dir.join(file)) {
                    Err(e) => panic!("fail to create file: {:?}", e),
                    Ok(_) => {}
                }
            }

            assert_eq!(
                case.expect,
                specify_makefile_name(tmp_dir.to_string_lossy().to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }
}
