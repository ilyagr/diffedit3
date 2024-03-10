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
          - type: Missing
          - type: Text
            value: added
          - type: Text
            value: added
        deleted_file:
          - type: Text
            value: deleted
          - type: Missing
          - type: Missing
        edited_file:
          - type: Text
            value: "First\nThird\nFourth\nFourthAndAHalf\n\nFifth\nSixth\n----\none two"
          - type: Text
            value: "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
          - type: Text
            value: "First\nSecond\nThird\nFifth\nSixth\n----\none\n"
        unsupported-left:
          - type: Unsupported
            value: demo of an unsupported file
          - type: Text
            value: text
          - type: Text
            value: text
        unsupported-right:
          - type: Text
            value: text
          - type: Unsupported
            value: demo of an unsupported file
          - type: Unsupported
            value: demo of an unsupported file
        "###);
    }
}
