#[cfg(target_os = "windows")]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(target_os = "windows")]
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
#[cfg(target_os = "windows")]
use winapi::um::securitybaseapi::GetTokenInformation;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{TokenElevation, TokenElevationType, TokenElevationTypeLimited, TOKEN_ELEVATION, TOKEN_ELEVATION_TYPE, TOKEN_QUERY};
#[cfg(target_os = "windows")]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};

#[derive(Default)]
pub struct WinElevationFunctions {}

impl WinElevationFunctions {
    #[cfg(target_os = "windows")]
    pub fn is_admin() -> bool {
        unsafe {
            let mut handle = INVALID_HANDLE_VALUE;
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) == 0 {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION {
                TokenIsElevated: 0,
            };
            let size = std::mem::size_of::<TOKEN_ELEVATION_TYPE>() as u32;
            let mut ret_size = 0;

            let result = GetTokenInformation(
                handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut ret_size,
            );

            if result == 0 {
                return false;
            }

            elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_admin() -> bool {
        true
    }

    #[cfg(target_os = "windows")]
    pub fn is_token_elevation_type_limited() -> Result<bool, u32> {
        unsafe {
            let mut token = INVALID_HANDLE_VALUE;
            let process = GetCurrentProcess();
    
            if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
                return Err(GetLastError());
            }
    
            let mut elevation_type_info = TOKEN_ELEVATION_TYPE::default();
            let mut return_length = 0;
    
            if GetTokenInformation(
                token,
                TokenElevationType,
                &mut elevation_type_info as *mut _ as *mut _,
                std::mem::size_of::<TOKEN_ELEVATION_TYPE>() as u32,
                &mut return_length,
            ) == 0
            {
                let error = GetLastError();
                CloseHandle(token);
                return Err(error);
            }
    
            CloseHandle(token);
    
            Ok(elevation_type_info == TokenElevationTypeLimited as u32)
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_token_elevation_type_limited() -> Result<bool, u32> {
        Ok(true)
    }
}

    // pub fn is_token_elevation_type_limited_x() -> Result<bool, u32> {
    //     unsafe {
    //         let mut token = INVALID_HANDLE_VALUE;
    //         let process = GetCurrentProcess();

    //         if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
    //             return Err(GetLastError());
    //         }
    
    //         let mut elevation_type = TOKEN_ELEVATION {
    //             TokenIsElevated: 0,
    //         };
    //         let mut return_length = 0;

    //         if GetTokenInformation(
    //             token,
    //             TokenElevation,
    //             &mut elevation_type as *mut _ as *mut _,
    //             std::mem::size_of::<TOKEN_ELEVATION_TYPE>() as u32,
    //             &mut return_length,
    //         ) == 0
    //         {
    //             return Err(GetLastError());
    //         }

    //         Ok(elevation_type == TokenElevationTypeLimited as u32)
    //     }
    // }
//}