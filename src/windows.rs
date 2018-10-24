use std::env;
use std::io::{self, Read, Write};
use std::mem;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};

use uuid::Uuid;

pub use std::os::windows::io::{FromRawSocket, RawSocket};
use winapi::ctypes::{c_uint, c_ulong, c_void};
use winapi::shared::ntdef::{HANDLE, NTSTATUS};
use winapi::um::processthreadsapi::GetCurrentProcessId;
use winapi::um::winsock2::{
    WSASocketW, FROM_PROTOCOL_INFO, WSAPROTOCOL_INFOW, WSA_FLAG_OVERLAPPED,
};

pub use self::RawSocket as FdType;

pub fn make_tcp_listener(fd: FdType) -> io::Result<TcpListener> {
    Ok(unsafe { FromRawSocket::from_raw_socket(fd) })
}

pub fn make_udp_socket(fd: FdType) -> io::Result<UdpSocket> {
    Ok(unsafe { FromRawSocket::from_raw_socket(fd) })
}

#[repr(C)]
pub struct IO_STATUS_BLOCK {
    status: *const c_void,
    information: *const c_ulong,
}

#[repr(C)]
pub struct FILE_INFORMATION {
    port: HANDLE,
    key: *const c_void,
}

#[link(name = "ntdll")]
extern "system" {
    #[link_name = "NtSetInformationFile"]
    fn NtSetInformationFile(
        file_handle: HANDLE,
        block: *mut IO_STATUS_BLOCK,
        file_info: *mut FILE_INFORMATION,
        len: c_ulong,
        cls: c_uint,
    ) -> NTSTATUS;
}

/// This detaches a socket from IOCP.
///
/// This only works on windows 8.1 and later.  In ealier windows
/// versions it's not possible to detach the socket until shutdown
/// which means that stuff that uses IOCP (like tokio) cannot be
/// used for reloading type scenarios.
unsafe fn detach_from_iocp(sock: FdType) {
    let mut status_block: IO_STATUS_BLOCK = mem::zeroed();
    let mut file_info: FILE_INFORMATION = mem::zeroed();

    // FileReplaceCompletionInformation
    NtSetInformationFile(
        sock as HANDLE,
        &mut status_block,
        &mut file_info,
        mem::size_of::<FILE_INFORMATION>() as c_ulong,
        61,
    );
}

pub fn get_fds() -> Option<Vec<FdType>> {
    let addr: SocketAddr = env::var("SYSTEMFD_SOCKET_SERVER")
        .ok()
        .and_then(|x| x.parse().ok())?;
    let secret: Uuid = env::var("SYSTEMFD_SOCKET_SECRET")
        .ok()
        .and_then(|x| x.parse().ok())?;

    env::remove_var("SYSTEMFD_SOCKET_SERVER");
    env::remove_var("SYSTEMFD_SOCKET_SECRET");

    let mut data = Vec::new();
    let proto_len = mem::size_of::<WSAPROTOCOL_INFOW>();

    let mut listener = TcpStream::connect(addr).ok()?;
    let pid = unsafe { GetCurrentProcessId() };
    listener
        .write_all(format!("{}|{}", secret, pid).as_bytes())
        .ok()?;
    listener.shutdown(Shutdown::Write).ok()?;
    listener.read_to_end(&mut data).ok()?;

    let items = data.len() / proto_len;
    let mut rv = Vec::new();

    for idx in 0..items {
        let offset = idx * proto_len;
        let proto_info: &mut WSAPROTOCOL_INFOW =
            unsafe { mem::transmute(data[offset..offset + proto_len].as_ptr()) };
        unsafe {
            let sock = WSASocketW(
                FROM_PROTOCOL_INFO,
                FROM_PROTOCOL_INFO,
                FROM_PROTOCOL_INFO,
                proto_info,
                0,
                WSA_FLAG_OVERLAPPED,
            ) as FdType;
            if sock == 0 {
                return None;
            }
            detach_from_iocp(sock);
            rv.push(sock);
        }
    }

    Some(rv)
}
