use fuse_rs::Filesystem;
use nix::fcntl::OFlag;

struct FuseFat32X {
    inner: crate::Fat32X,
}
impl Filesystem for FuseFat32X {
    fn open(&mut self, path: &std::path::Path, file_info: &mut fuse_rs::fs::OpenFileInfo) -> fuse_rs::Result<()> {
        let read = file_info.flags().map(|x| x.contains(OFlag::O_RDONLY) || x.contains(OFlag::O_RDWR)).unwrap_or_default();
        let write = file_info.flags().map(|x| x.contains(OFlag::O_WRONLY) || x.contains(OFlag::O_RDWR)).unwrap_or_default();
        let append = file_info.flags().map(|x| x.contains(OFlag::O_APPEND)).unwrap_or_default();
        let create = file_info.flags().map(|x| x.contains(OFlag::O_CREAT)).unwrap_or_default();
        todo!()
    }
}
