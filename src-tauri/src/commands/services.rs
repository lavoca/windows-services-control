
use windows::{
    Win32::System::Services::{
            CloseServiceHandle, ControlService, ENUM_SERVICE_STATUS_PROCESSW, ENUM_SERVICE_TYPE, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW, QueryServiceStatus, SC_ENUM_PROCESS_INFO, SC_HANDLE, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_CONTROL_CONTINUE, SERVICE_CONTROL_PAUSE, SERVICE_CONTROL_STOP, SERVICE_DRIVER, SERVICE_FILE_SYSTEM_DRIVER, SERVICE_KERNEL_DRIVER, SERVICE_PAUSE_CONTINUE, SERVICE_QUERY_STATUS, SERVICE_START, SERVICE_STATE_ALL, SERVICE_STATUS, SERVICE_STATUS_PROCESS, SERVICE_STOP, SERVICE_STOPPED, SERVICE_USER_OWN_PROCESS, SERVICE_USER_SHARE_PROCESS, SERVICE_WIN32, SERVICE_WIN32_OWN_PROCESS, SERVICE_WIN32_SHARE_PROCESS, StartServiceW
        }, 
    core::{HSTRING, PWSTR}
};

use tauri::command;
use serde::{Serialize, Deserialize};


unsafe fn pwstr_to_string(pwstr: PWSTR) -> Option<String> {

    if pwstr.is_null(){
        return None; // if pwstr is null we return None
    }    

    let owned_string = pwstr.to_string() // convert to string
    .ok()
    .filter(|s| !s.is_empty()); // check if the string is empty and returns None if it is

    // CoTaskMemFree(Some(pwstr.as_ptr() as _)); // <- DO NOT FREE MEMORY WE DONT OWN !!!!
    // the pwstr that comes to this function is from a Rust vec<> that will go out of scope and we shouldnt free it manually rust will do it 
    // only use CoTaskMemFree for data that was allocated by windows or us by using CoTaskMemAlloc  
    owned_string
     
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub service_type: String,
    pub can_interact: bool
}

struct QueryInfo {
    current_state: String,
    string_type: String,
    enum_type: ENUM_SERVICE_TYPE,
    service_status: SERVICE_STATUS,
    can_interactb: bool
}

pub struct ServiceManager {
    scm_handle: SC_HANDLE,
}

pub struct OpenService {
    service_handle: SC_HANDLE,
}

impl ServiceManager {
    pub fn new() -> windows_core::Result<Self> {

        unsafe {
            let scm_handle: SC_HANDLE = OpenSCManagerW(None, None,     SC_MANAGER_ENUMERATE_SERVICE)?;  
            Ok(Self { scm_handle })
        }
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        unsafe{
            let _ = CloseServiceHandle(self.scm_handle);
        }
    }
}

impl OpenService {   // &str as a function param accepts both String and str              
    pub fn new(sc_manager: SC_HANDLE, service_name: &str, accesse_rights: u32) -> windows_core::Result<Self> {
        unsafe{
            let service_hstring = HSTRING::from(service_name);
            let service_handle = OpenServiceW(sc_manager, &service_hstring, accesse_rights)?;
            Ok(Self{service_handle})
        }
    }
}

impl Drop for OpenService {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseServiceHandle(self.service_handle);
        }
    }
}


#[command]
pub fn enumerate_services() -> Result<Vec<ServiceInfo>, String> {

    unsafe {

        let manager = ServiceManager::new().map_err(|e| format!("Failed to open service manager: {}", e))?;

        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;
        let mut resume_handle: u32 =0;

        // first call to get buffer size 
    let _ = EnumServicesStatusExW (
            manager.scm_handle, 
            SC_ENUM_PROCESS_INFO, 
            SERVICE_WIN32 | SERVICE_DRIVER, 
            SERVICE_STATE_ALL, 
            None, 
            &mut bytes_needed, // the size we will use to create a buffer 
            &mut services_returned, 
            Some(&mut resume_handle), 
            None);

        // create the buffer with the size we got above    
        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];

        // secon call to populate the buffer with services info 
        EnumServicesStatusExW(manager.scm_handle, 
            SC_ENUM_PROCESS_INFO, 
            SERVICE_WIN32 | SERVICE_DRIVER, 
            SERVICE_STATE_ALL, 
            Some(&mut buffer), 
            &mut bytes_needed,  
            &mut services_returned, // number of services found
            Some(&mut resume_handle), 
            None).map_err(|e| e.to_string())?;


        // EnumServicesStatusExW() populates the buffer with raw bytes of info about the services and we need to convert them to pointer to ENUM_SERVICE_STATUS_PROCESSW structs 
        // that windows understands that we will once agin convert to a type that rust understands this time essentialy making ENUM_SERVICE_STATUS_PROCESSW like a middle man
        let pointer_to_services = buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW;


        // to convert it to rust friendly array we convert it to a slice with from_raw_parts() so we can iterate over the info inside.
// ENUM_SERVICE_STATUS_PROCESSW is a raw pointer to first structs in an array of structs => convert to rust slice containing all the C structs => converting every C struct into ServiceInfo struct
        let services_slice = std::slice::from_raw_parts( // A slice is a reference to part of an array or vector. It doesn’t own the data; it just points to the same memory and has a length.
            pointer_to_services, // Pointer to first element
            services_returned as usize); // Number of elements in array


        // vector to hold rust structs.
        let mut services_info = Vec::new();
        
        // iterate over slice holding C structs and convert every one to rust struct and push them to the vector
        for service in services_slice {

            let service_name = match pwstr_to_string(service.lpServiceName) {
                Some(name) => name,
                None => String::from("unknown")
            };

            // this gets the service state and type info 
            let query_info = query_service_state(&service_name).map_err(|e| e.to_string())?;

            // get the service type as an ENUM_SERVICE_TYPE and not a String from query_service_state() 
            let enum_service_type = query_info.enum_type;

            let interact = match enum_service_type {
                SERVICE_WIN32_OWN_PROCESS | SERVICE_WIN32_SHARE_PROCESS => true,
                _ => false
            };

            services_info.push(ServiceInfo {

                // the data here from the slice is of PWSTR type and we convert it to a rust string
                name: service_name, 
                display_name: match pwstr_to_string(service.lpDisplayName){
                    Some(name) => name,
                    None => String::from("unknown")
                },

                // the data here from the slice is wrapped with a C type but has a u32 inside and we access that iner value with .0
                service_type: query_info.string_type,
                status: query_info.current_state,
                can_interact: interact
            });
        }
        println!("services: {:?}", services_info);
        Ok(services_info)


    }
    
}





fn query_service_state(service_name: &str) -> windows_core::Result<QueryInfo> {
    unsafe {

        let scm = ServiceManager::new()?;
        let service = OpenService::new(scm.scm_handle, service_name, SERVICE_QUERY_STATUS)?;
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(service.service_handle, &mut status)?;
        // Extract the inner u32 value from the wrapper type
        let state = match status.dwCurrentState.0 {
            1 => "Stopped",           // SERVICE_STOPPED
            2 => "Start Pending",     // SERVICE_START_PENDING  
            3 => "Stop Pending",      // SERVICE_STOP_PENDING
            4 => "Running",           // SERVICE_RUNNING
            5 => "Continue Pending",  // SERVICE_CONTINUE_PENDING
            6 => "Pause Pending",     // SERVICE_PAUSE_PENDING
            7 => "Paused",            // SERVICE_PAUSED
            _ => "Unknown",
        }.to_string();

        let mut string_type = String::from("unknown"); // service type as string to be used in the frontend
        let mut ser_type = status.dwServiceType; // service type to be used in the backend
        let flag = status.dwServiceType.0; // used to calculate what type a service is 
        
        let types = vec![
            (SERVICE_FILE_SYSTEM_DRIVER, "File System Driver"), 
            (SERVICE_KERNEL_DRIVER, "Kernel Driver"), 
            (SERVICE_WIN32_OWN_PROCESS, "Win32 Own Process"), 
            (SERVICE_WIN32_SHARE_PROCESS, "Win32 Share Process"), 
            (SERVICE_USER_OWN_PROCESS, "User Own Process"), 
            (SERVICE_USER_SHARE_PROCESS, "User Share Process")];


        for (bits, description) in types {
            if flag & bits.0 != 0 { // we need to do this "bit &" to get the actual types
                string_type = description.to_string();
                ser_type = bits;
                break
            } 
        }

        let interact = match ser_type { // if its a win32 service type then user is allowed to stop it if it is not then it is dangerous to give control to user
            SERVICE_WIN32_OWN_PROCESS | SERVICE_WIN32_SHARE_PROCESS => true,
            _ => false
            };

        Ok(QueryInfo{
            current_state: state,
            string_type: string_type, // for frontend 
            enum_type: ser_type, // for backend
            service_status: status, // so we dont repeat "let mut status = SERVICE_STATUS::default()" every function
            can_interactb: interact // bool value used in backend to decide if a services should be stoped
        })
    }
}




#[command]
pub fn stop_service(service_name: &str) -> Result<(), String> {
    unsafe {
        let scm = ServiceManager::new().map_err(|e| e.to_string())?; // CloseServiceHandle() is called automatically because of the implementation above
        let service = OpenServiceW(
            scm.scm_handle, 
            &HSTRING::from(service_name), // OpenServiceW expects PCWSTR and PCWSTR accepts HSTRING 
            SERVICE_STOP

        ).map_err(|e| e.to_string())?;

        
        let query = query_service_state(service_name).map_err(|e| e.to_string())?; 
        // get service status of type SERVICE_STATUS from query function
        let mut status = query.service_status; // make this mut becasue we are passing it to ControlService which expects it to be pointer to mut


        /* 
        let status_ptr = &mut status; // convert it to a pointer
        let status_raw_ptr = status_ptr as *mut SERVICE_STATUS; // convert the pointer to a raw pointer
        */


        if query.can_interactb {
            // this method expects the status as a raw pointer but &mut also works and is safer for rust so no need to convert it to a raw pointer
            ControlService(service, SERVICE_CONTROL_STOP, &mut status ).map_err(|e| e.to_string())?; 
            CloseServiceHandle(service).map_err(|e| e.to_string())?;
        }
       
        Ok(())
    }
}

#[command]
pub fn start_service(service_name: &str) -> Result<(), String> {
    unsafe{
        let scm = ServiceManager::new().map_err(|e| e.to_string())?;
        // see new() function in "impl OpenService" to see the original method we also used it directly in stop_service()
        let service = OpenService::new(scm.scm_handle, service_name, SERVICE_START).map_err(|e| e.to_string())?; 

        let query = query_service_state(service_name).map_err(|e| e.to_string())?;

        if query.can_interactb {
            StartServiceW(service.service_handle, None).map_err(|e| e.to_string())?;
        }


        Ok(())
    }
}

#[command] 
pub fn pause_service(service_name: &str) -> Result<(), String> {
    unsafe {
        let scm = ServiceManager::new().map_err(|e| e.to_string())?;
        let service = OpenService::new(scm.scm_handle, service_name, SERVICE_PAUSE_CONTINUE).map_err(|e| e.to_string())?;
        let query = query_service_state(service_name).map_err(|e| e.to_string())?;
        let mut status = query.service_status;
        if query.can_interactb {
            ControlService(service.service_handle, SERVICE_CONTROL_PAUSE, &mut status).map_err(|e| e.to_string())?; // &mut also works instead of a raw pointer
        }
        Ok(())
    }
}

#[command] 
pub fn resume_service(service_name: &str) -> Result<(), String> {
    unsafe {
        let scm = ServiceManager::new().map_err(|e| e.to_string())?;
        let service = OpenService::new(scm.scm_handle, service_name, SERVICE_PAUSE_CONTINUE).map_err(|e| e.to_string())?;
        let query = query_service_state(service_name).map_err(|e| e.to_string())?;
        let mut status = query.service_status;
        if query.can_interactb {
            //let status_ptr = &mut status;
            //let status_raw_ptr = status_ptr as *mut SERVICE_STATUS;
            ControlService(service.service_handle, SERVICE_CONTROL_CONTINUE, &mut status).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}


