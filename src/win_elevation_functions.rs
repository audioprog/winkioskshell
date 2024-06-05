use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TokenElevation, TokenElevationType, TokenElevationTypeLimited, TOKEN_ELEVATION, TOKEN_ELEVATION_TYPE, TOKEN_QUERY};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};


#[derive(Default)]
pub struct WinElevationFunctions {
}

impl WinElevationFunctions {
    pub fn is_admin() -> bool {
        unsafe {
            let mut handle = INVALID_HANDLE_VALUE;
            if OpenProcessToken(winapi::um::processthreadsapi::GetCurrentProcess(), TOKEN_QUERY, &mut handle) == 0 {
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
}