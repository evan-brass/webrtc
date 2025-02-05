#[cxx::bridge]
pub mod ffi {
	extern "C++" {
		include!("api/create_peerconnection_factory.h");
		#[namespace = "rtc"]
		type Thread;
	}
}
