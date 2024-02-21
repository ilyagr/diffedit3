pub mod fs;
pub mod types;
pub mod local_server;

pub use fs::{Cli, ThreeDirInput};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_yaml_snapshot!(ThreeDirInput::FakeData.scan().unwrap(), 
        @r###"
        ---
        added file:
          - ~
          - added
          - added
        deleted_file:
          - deleted
          - ~
          - ~
        edited_file:
          - "First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\none two"
          - "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
          - "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
        "###);
    }
}
