use bincode;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// serialize and write the given object with a trailing newline character.
pub fn write_serialized<S, W>(s: S, w: &mut W) -> bincode::Result<()>
where
    S: Serialize,
    W: Write,
{
    w.write(&bincode::serialize(&s)?)?;
    w.write(&"\n".as_bytes())?;
    Ok(())
}

/// deserialize object from a byte array with a trailing newline character.
pub fn read_serialized<'de, D>(bytes: &'de [u8]) -> bincode::Result<D>
where
    D: Deserialize<'de>,
{
    let result = bincode::deserialize::<'de, D>(&bytes)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestStruct {
        a: usize,
        b: bool,
    }

    #[test]
    fn serialize_and_deserialize() {
        let original = TestStruct { a: 12345, b: true };
        let mut buffer = Vec::<u8>::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        write_serialized(original.clone(), &mut cursor).expect("write_serialized failed");
        let reconstructed: TestStruct =
            read_serialized(&buffer[..]).expect("read_serialized failed");
        println!("{:?}", buffer);
        assert_eq!(original, reconstructed);
    }
}
