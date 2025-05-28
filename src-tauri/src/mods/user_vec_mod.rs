use std::error::Error;

#[derive(Debug, Clone)]
pub struct UserVecU8 {
    datas: Vec<u8>,
    max_length: usize,
}
impl UserVecU8 {
    /// 建立 UserVecU8，設定最大容量<br>
    /// Creates a new UserVecU8 with specified maximum capacity
    pub fn new(max_length: usize) -> Self {
        Self {
            datas: Vec::new(),
            max_length,
        }
    }

    /// 取得最大容量<br>
    /// Returns the maximum capacity
    pub fn get_max_len(&self) -> usize {
        self.max_length
    }

    /// 取得所有目前儲存的資料副本<br>
    /// Returns a clone of all currently stored data
    pub fn show(&self) -> Vec<u8> {
        self.datas.clone()
    }

    /// 取得目前儲存資料的長度<br>
    /// Returns the current number of elements stored
    pub fn len(&self) -> usize {
        self.datas.len()
    }

    /// 是否已經沒有任何元素<br>
    /// Returns true if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.datas.is_empty()
    }

    /// 是否已經達到最大長度<br>
    /// Returns true if the vector has reached its maximum capacity
    pub fn is_full(&self) -> bool {
        self.datas.len() >= self.max_length
    }

    /// 清空所有資料<br>
    /// Clears all elements from the vector
    pub fn clear(&mut self) {
        self.datas.clear();
    }

    /// 嘗試推入一個元素；若未超過最大長度則成功，否則回傳 Err<br>
    /// Attempts to push a value; succeeds if under max length, otherwise returns Err
    pub fn push(&mut self, value: u8) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.datas.len() < self.max_length {
            self.datas.push(value);
            Ok(())
        } else {
            Err("Data is full".into())
        }
    }

    /// 延伸序列新增多個元素；若延伸後長度超過最大長度則回傳 Err<br>
    /// Extends the vector with multiple elements; returns Err if exceeding max length
    pub fn extend<T: AsRef<[u8]>>(&mut self, data: T) -> Result<(), Box<dyn Error + Send + Sync>> {
        let slice = data.as_ref();
        if self.len() + slice.len() > self.max_length {
            return Err("Data is full".into());
        }
        self.datas.extend_from_slice(slice);
        Ok(())
    }
}
