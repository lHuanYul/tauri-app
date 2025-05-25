use std::error::Error;

#[derive(Debug, Clone)]
pub struct UserVecU8 {
    datas: Vec<u8>,
    max_length: usize,
}
impl UserVecU8 {
    pub fn new(max_length: usize) -> Self {
        Self {
            datas: Vec::new(),
            max_length,
        }
    }

    pub fn get_max_len(&self) -> usize {
        self.max_length
    }

    pub fn show(&self) -> Vec<u8> {
        self.datas.clone()
    }

    pub fn len(&self) -> usize {
        self.datas.len()
    }

    /// 是否已經沒有任何元素
    pub fn is_empty(&self) -> bool {
        self.datas.is_empty()
    }

    /// 是否已經達到最大長度
    pub fn is_full(&self) -> bool {
        self.datas.len() >= self.max_length
    }

    /// 清空所有資料
    pub fn clear(&mut self) {
        self.datas.clear();
    }

    /// 嘗試推入一個元素，若已滿則回傳 Err
    pub fn push(&mut self, value: u8) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.datas.len() < self.max_length {
            self.datas.push(value);
            Ok(())
        } else {
            Err("已達到最大長度，無法再 push".into())
        }
    }

    pub fn extend<T: AsRef<[u8]>>(&mut self, data: T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let slice = data.as_ref();
        if self.len() + slice.len() > self.max_length {
            return Err("延伸後超過最大長度，操作失敗".into());
        }
        self.datas.extend_from_slice(slice);
        Ok(())
    }
}
