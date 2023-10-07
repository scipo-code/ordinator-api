trait FromRawData {
    fn from_raw(data: RawData) -> Result<Self, Error>;
}
