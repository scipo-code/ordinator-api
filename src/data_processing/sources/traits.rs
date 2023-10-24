trait FromRawData {
    fn from_raw_excel(data: RawData) -> Result<Self, Error>;


}
