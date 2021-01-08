use alloc::{rssalloc::RSSAllocator, Allocator, Block};
pub use error::ErrorKind;
use mount::MountValidator;
use ops::Ops;

use std::cmp;
use std::fs::{File, OpenOptions};
use std::path::Path;

pub const NEONDB_FILE_EXT: &str = "neondb";
pub const NEONDB_FILE_MARK: &str = "A NeonDB Volume!";
pub const NEONDB_FILE_SIZE: u64 = 1 << 23;

// Setiap volume memiliki string NEONDB_FILE_MARK di beberapa byte awal,
// dimana byte string tersebut tidak boleh diubah-ubah.
pub const NEONDB_FILE_ALLOCATABLE_START: u64 = NEONDB_FILE_MARK.len() as u64;
pub const NEONDB_FILE_ALLOCATABLE_SIZE: u64 = NEONDB_FILE_SIZE - NEONDB_FILE_MARK.len() as u64;

mod alloc;
mod error;
mod mount;
mod ops;

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, self::error::ErrorKind>;

/// Public API dari package storage.
///
/// Package ini hanya menyediakan hal-hal terkait penyimpanan secara
/// minimalis (low-level). Untuk implementasi detail seperti kepemilikan
/// dari sebuah block dan sebagainya, dilakukan oleh pengguna sendiri.
///
/// # Examples
///
/// ```no_run
/// use storage::Storage;
/// use std::path::Path;
///
/// let mut s = Storage::new();
/// let path = Path::new("path-ke-volume.neondb");
///
/// s.mount(path).unwrap();
///
/// let bytes = "sesuatu".as_bytes();
/// let addr = s.alloc(200).unwrap();
///
/// s.write(addr, &bytes).unwrap();       // ok
/// s.write(addr + 50, &bytes).unwrap();  // ok
///
/// s.write(addr + 999, &bytes).unwrap(); // ilegal, error
/// ```
///
pub struct Storage {
    volume: Option<File>,
    allocator: Box<dyn Allocator>,

    // cache untuk informasi dari blok yang ada di volume
    blocks_cache: Vec<Block>,
    need_to_refresh_cache: bool,
}

impl Storage {
    /// Membuat instance baru dari storage.
    ///
    /// Penggunaan sebelum melakukan mounting volume dari penyimpanan
    /// akan menghasilkan error.
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::Storage;
    ///
    /// let mut s = Storage::new();
    /// ```
    pub fn new() -> Storage {
        Storage {
            volume: None,
            allocator: Box::new(RSSAllocator::new()),
            blocks_cache: Vec::new(),
            need_to_refresh_cache: true,
        }
    }

    /// Melakukan mounting (atau memasang) sebuah volume yang menjadi
    /// media penyimpanan data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    /// ```
    pub fn mount(&mut self, path: &Path) -> Result<()> {
        MountValidator::validate(path)?;

        self.volume = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|_| panic!("internal error"))
            .ok();

        self.allocator.init(self.volume.as_mut().unwrap())?;

        self.need_to_refresh_cache = true;
        Ok(())
    }

    /// Membuat volume baru dengan nama path yang diberikan, menginisialisasi,
    /// sekaligus melakukan mounting terhadap volume tersebut.
    ///
    /// Method ini akan menghasilkan error jika volume dengan nama path yang
    /// diberikan sudah ada sebelumnya.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("vol-yang-belum-ada.neondb");
    ///
    /// s.mount_new(vol).unwrap();
    /// ```
    pub fn mount_new(&mut self, path: &Path) -> Result<()> {
        MountValidator::validate_new(path)?;

        self.volume = mount::new_volume(path)
            .map_err(|_| panic!("internal error"))
            .ok();
        self.allocator.init_new(self.volume.as_mut().unwrap())?;

        self.need_to_refresh_cache = true;
        Ok(())
    }

    /// Melakukan unmounting (atau melepas) volume penyimpanan yang sedang
    /// digunakan.
    ///
    /// Error jika belum ada volume yang di-mounting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// // blah blah blah
    ///
    /// s.unmount().unwrap();
    /// ```
    pub fn unmount(&mut self) -> Result<()> {
        if self.volume.is_none() {
            return Err(ErrorKind::VolumeNotFound);
        }
        self.volume = None;
        self.allocator.reset();
        self.need_to_refresh_cache = true;

        Ok(())
    }

    /// Melakukan operasi read pada address tertentu, dan menyimpan
    /// hasilnya ke dalam buff.
    ///
    /// Nilai yang dikembalikan adalah jumlah byte yang berhasil dibaca.
    /// Namun, terdapat kemungkinan bahwa jumlah bytes yang dibaca tidak
    /// sama dengan ukuran dari buffer bytes yang diberikan. (Pada kasus
    /// ini, method akan tetap melakukan pembacaan sebisanya selama tidak
    /// terjadi pengaksesan terhadap address yang ilegal).
    ///
    /// Method ini akan menghasilkan error jika belum ada volume yang
    /// dimounting, atau operasi pembacaan dilakukan pada address yang
    /// ilegal (belum dialokasikan).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    ///
    /// let mut buff = [0u8; 10];
    /// let addr = 100;     // asumsikan blok pada alamat ini sudah
    ///                     // dialokasikan sebelumnya
    ///
    /// let res = s.read(addr, &mut buff);
    ///
    /// if let Ok(size) = res {
    ///     // pembacaan sukses
    /// } else {
    ///     // error! handle di sini
    /// }
    /// ```
    pub fn read(&mut self, address: u64, buff: &mut [u8]) -> Result<usize> {
        let max_len = Ops::max_operation_len_at(address, self.blocks().unwrap())?;
        let len = cmp::min(max_len, buff.len());

        Ok(Ops::read(
            address,
            &mut buff[..len],
            self.volume.as_mut().unwrap(),
        ))
    }

    /// Melakukan operasi write pada address tertentu, dengan menggunakan
    /// byte-byte yang terdapat pada buff.
    ///
    /// Nilai yang dikembalikan adalah jumlah byte yang berhasil ditulis.
    /// Namun, terdapat kemungkinan bahwa jumlah bytes yang ditulis tidak
    /// sama dengan ukuran dari buffer bytes yang diberikan. (Pada kasus
    /// ini, method akan tetap melakukan penulisan sebisanya selama tidak
    /// terjadi pengaksesan terhadap address yang ilegal).
    ///
    /// Method ini akan menghasilkan error jika belum ada volume yang
    /// dimounting, atau operasi penulisan dilakukan pada address yang
    /// ilegal (belum dialokasikan).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    ///
    /// let buff = [65u8; 10];
    /// let addr = 100;     // asumsikan blok pada alamat ini sudah
    ///                     // dialokasikan sebelumnya
    ///
    /// let res = s.write(addr, &buff);
    ///
    /// if let Ok(size) = res {
    ///     // penulisan sukses
    /// } else {
    ///     // error! handle di sini
    /// }
    /// ```
    pub fn write(&mut self, address: u64, buff: &[u8]) -> Result<usize> {
        let max_len = Ops::max_operation_len_at(address, self.blocks().unwrap())?;
        let len = cmp::min(max_len, buff.len());

        Ok(Ops::write(
            address,
            &buff[..len],
            self.volume.as_mut().unwrap(),
        ))
    }

    /// Mengalokasikan sebuah blok dengan ukuran yang diberikan.
    ///
    /// Nilai yang dikembalikan adalah alamat dari blok yang baru
    /// dialokasikan.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    ///
    /// let res = s.alloc(100);     // alokasi 100 bytes
    ///
    /// if let Ok(addr) = res {
    ///    // alokasi berhasil, lakukan sesuatu terhadap addr
    /// } else {
    ///    // alokasi gagal
    /// }
    /// ```
    pub fn alloc(&mut self, size: usize) -> Result<u64> {
        if self.volume.is_none() {
            return Err(ErrorKind::VolumeNotFound);
        }

        self.allocator
            .alloc(self.volume.as_mut().unwrap(), size)
            .and_then(|address| {
                self.need_to_refresh_cache = true;
                Ok(address)
            })
    }

    /// Men-dealokasi-kan sebuah blok yang terletak pada alamat
    /// yang diberikan.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    ///
    /// // blah blah blah
    ///
    /// let res = s.dealloc(60);    // dealokasi blok pada alamat 60
    ///
    /// if let Err(err) = res {
    ///     // handle error di sini
    /// }
    /// ```
    pub fn dealloc(&mut self, address: u64) -> Result<()> {
        if self.volume.is_none() {
            return Err(ErrorKind::VolumeNotFound);
        }

        self.allocator
            .dealloc(self.volume.as_mut().unwrap(), address)
            .and_then(|_| {
                self.need_to_refresh_cache = true;
                Ok(())
            })
    }

    /// Mendapatkan informasi terkait blok-blok yang terdapat di dalam
    /// volume yang sedang dimounting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::Storage;
    /// use std::path::Path;
    ///
    /// let mut s = Storage::new();
    /// let vol = Path::new("path-ke-volume.neondb");
    ///
    /// s.mount(vol).unwrap();
    ///
    /// let blocks = s.blocks().unwrap();
    /// ```
    pub fn blocks(&mut self) -> Result<&[Block]> {
        if self.volume.is_none() {
            return Err(ErrorKind::VolumeNotFound);
        }

        if self.need_to_refresh_cache {
            self.blocks_cache = self.allocator.blocks(self.volume.as_mut().unwrap());
            self.need_to_refresh_cache = false;
        }

        Ok(&self.blocks_cache)
    }
}
