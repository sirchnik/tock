// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! PSC3 flash integration through the boot ROM flash function table.

use core::cell::Cell;
use core::ptr;

use kernel::utilities::StaticRef;
use spe::hil::flash::Flash as FlashSpeHil;
use spe::ErrorCode;

const CYBOOT_FLASH_SECTOR_SIZE: usize = 128 * 1024;
const CYBOOT_N_FLASH_SECTORS: usize = 2;
const CY_FLASH_ROW_SIZE: usize = 512;
const CYBOOT_FLASH_COLUMN33_SIZE: usize = 16;

const CYBOOT_FLASH_TOTAL_SIZE: usize = CYBOOT_FLASH_SECTOR_SIZE * CYBOOT_N_FLASH_SECTORS;
const CYBOOT_FLASH_BASE_ADDR: usize = 0x3200_0000;

#[repr(u32)]
enum CybootFlashMode {
    Blocking = 1 << 0,
}

#[repr(u32)]
enum CybootFlashStatus {
    Success = 0x0D50_B002,
    Failed = 0xBAF0_0008,
    RecoverErr = 0xBAF0_0009,
    InitFailed = 0xBAF0_004B,
    AddrInvalid = 0xBAF0_004C,
    ParamInvalid = 0xBAF0_004D,
    FaultUnexpected = 0xBAF0_004E,
}

const ROM_FUNC_ADDR: usize = 0x1080_FF6C;
const FLASH_ROW_AND_COL33_WORDS: usize = (CY_FLASH_ROW_SIZE + CYBOOT_FLASH_COLUMN33_SIZE) / 4;
const SECTOR_INDEX_WITHOUT_SFLASH: u32 = 0;

const ROM_FUNC: StaticRef<RomFunctionList> =
    // ### Safety
    // ROM_FUNC_ADDR is a fixed PSC3 ROM address containing the flash function table.
    unsafe { StaticRef::new(ROM_FUNC_ADDR as *const RomFunctionList) };

#[repr(C)]
struct CybootFlashRefresh {
    min_count: u32,
    max_count: u32,
    min_page_addr: u32,
    scratch_row_idx: u32,
}

type CybootFlashCallback = unsafe extern "C" fn(*mut ());

unsafe extern "C" fn flash_noop_callback(_param: *mut ()) {}

#[repr(C)]
struct CybootFlashContext {
    flags: u32,
    hv_params_addr: u32,
    refresh: *mut CybootFlashRefresh,
    callback_pre_irq: CybootFlashCallback,
    callback_post_irq: CybootFlashCallback,
    callback_complete: CybootFlashCallback,
    callback_param: u32,
    state: u32,
    flash_addr: u32,
    data_addr: u32,
    reserved: [u32; 2],
}

impl CybootFlashContext {
    const fn blocking_default() -> Self {
        Self {
            flags: CybootFlashMode::Blocking as u32,
            hv_params_addr: 0,
            refresh: ptr::null_mut(),
            callback_pre_irq: flash_noop_callback,
            callback_post_irq: flash_noop_callback,
            callback_complete: flash_noop_callback,
            callback_param: 0,
            state: 0,
            flash_addr: 0,
            data_addr: 0,
            reserved: [0; 2],
        }
    }
}

type FlashEraseSubSectorFn = unsafe extern "C" fn(u32, *mut CybootFlashContext) -> u32;
type FlashEraseSectorFn = unsafe extern "C" fn(u32, *mut CybootFlashContext) -> u32;
type FlashEraseAllFn = unsafe extern "C" fn(u32, *mut CybootFlashContext) -> u32;
type FlashProgramSubSectorFn = unsafe extern "C" fn(u32, u32, *mut CybootFlashContext) -> u32;
type FlashProgramSectorFn = unsafe extern "C" fn(u32, u32, *mut CybootFlashContext) -> u32;
type FlashProgramAllFn = unsafe extern "C" fn(u32, u32, *mut CybootFlashContext) -> u32;
type FlashMarginModeScreenFn =
    unsafe extern "C" fn(u32, u32, u32, *mut u32, *mut CybootFlashContext) -> u32;

type FlashRefreshInitFn = unsafe extern "C" fn(u32, *mut CybootFlashRefresh);
type FlashRefreshInitAllFn = unsafe extern "C" fn(*mut CybootFlashRefresh);
type FlashRefreshTestFn = unsafe extern "C" fn(*const CybootFlashRefresh) -> bool;
type FlashRefreshPerformFn = unsafe extern "C" fn(u32, *mut CybootFlashRefresh) -> u32;
type FlashRefreshPerformStartFn =
    unsafe extern "C" fn(u32, *mut CybootFlashRefresh, *const CybootFlashContext) -> u32;
type FlashRefreshRecoverFn = unsafe extern "C" fn(u32, *mut CybootFlashRefresh) -> u32;
type FlashRefreshRecoverAllFn = unsafe extern "C" fn(*mut CybootFlashRefresh) -> u32;

type FlashReadScratchCol33Fn = unsafe extern "C" fn(u32, u32, *mut u32, *mut u32) -> u32;
type FlashReadMainCol33Fn = unsafe extern "C" fn(u32, u32, *mut u32) -> u32;
type FlashRefreshGetScratchIdxFn = unsafe extern "C" fn(u32, *mut CybootFlashRefresh);
type FlashRefreshGetMinMaxFn = unsafe extern "C" fn(u32, *mut CybootFlashRefresh);
type FlashRefreshGetSectorIdxFn = unsafe extern "C" fn(u32, *mut u32, *mut u32) -> u32;

type FlashOperationFn = unsafe extern "C" fn(u32, u32, *const u32, *mut CybootFlashContext) -> u32;
type FlashCompleteFn = unsafe extern "C" fn(*const CybootFlashContext);
type IsFlashReadyFn = unsafe extern "C" fn(*const CybootFlashContext) -> bool;
type FlashIrqHandlerFn = unsafe extern "C" fn();

type FlashEraseRowFn = unsafe extern "C" fn(u32, *mut CybootFlashContext) -> u32;
type FlashProgramRowFn = unsafe extern "C" fn(u32, *const u32, *mut CybootFlashContext) -> u32;
type FlashWriteRowFn = unsafe extern "C" fn(u32, *const u32, *mut CybootFlashContext) -> u32;

type FlashEraseRowStartFn = unsafe extern "C" fn(u32, *mut CybootFlashContext) -> u32;
type FlashProgramRowStartFn = unsafe extern "C" fn(u32, *const u32, *mut CybootFlashContext) -> u32;
type FlashWriteRowStartFn = unsafe extern "C" fn(u32, *const u32, *mut CybootFlashContext) -> u32;

#[repr(C)]
struct RomFunctionList {
    cyboot_flash_erase_sub_sector: FlashEraseSubSectorFn,
    cyboot_flash_erase_sector: FlashEraseSectorFn,
    cyboot_flash_erase_all: FlashEraseAllFn,
    cyboot_flash_program_sub_sector: FlashProgramSubSectorFn,
    cyboot_flash_program_sector: FlashProgramSectorFn,
    cyboot_flash_program_all: FlashProgramAllFn,
    cyboot_flash_margin_mode_screen: FlashMarginModeScreenFn,

    cyboot_flash_refresh_init: FlashRefreshInitFn,
    cyboot_flash_refresh_init_all: FlashRefreshInitAllFn,
    cyboot_flash_refresh_test: FlashRefreshTestFn,
    cyboot_flash_refresh_perform: FlashRefreshPerformFn,
    cyboot_flash_refresh_perform_start: FlashRefreshPerformStartFn,
    cyboot_flash_refresh_recover: FlashRefreshRecoverFn,
    cyboot_flash_refresh_recover_all: FlashRefreshRecoverAllFn,
    cyboot_flash_read_scratch_col33: FlashReadScratchCol33Fn,
    cyboot_flash_read_main_col33: FlashReadMainCol33Fn,
    cyboot_flash_refresh_get_scratch_idx: FlashRefreshGetScratchIdxFn,
    cyboot_flash_refresh_get_min_max: FlashRefreshGetMinMaxFn,
    cyboot_flash_refresh_get_sector_idx: FlashRefreshGetSectorIdxFn,

    unused_2: [u32; 6],

    cyboot_flash_operation: FlashOperationFn,
    cyboot_flash_complete: FlashCompleteFn,
    cyboot_is_flash_ready: IsFlashReadyFn,
    cyboot_flash_irq_handler: FlashIrqHandlerFn,

    cyboot_flash_erase_row: FlashEraseRowFn,
    cyboot_flash_program_row: FlashProgramRowFn,
    cyboot_flash_write_row: FlashWriteRowFn,

    cyboot_flash_erase_row_start: FlashEraseRowStartFn,
    cyboot_flash_program_row_start: FlashProgramRowStartFn,
    cyboot_flash_write_row_start: FlashWriteRowStartFn,
}

fn map_rom_status(code: u32) -> Result<(), ErrorCode> {
    match code {
        x if x == CybootFlashStatus::Success as u32 => Ok(()),
        x if x == CybootFlashStatus::AddrInvalid as u32
            || x == CybootFlashStatus::ParamInvalid as u32 =>
        {
            Err(ErrorCode::InvalidArgument)
        }
        x if x == CybootFlashStatus::Failed as u32
            || x == CybootFlashStatus::RecoverErr as u32
            || x == CybootFlashStatus::InitFailed as u32
            || x == CybootFlashStatus::FaultUnexpected as u32 =>
        {
            Err(ErrorCode::GenericError)
        }
        _ => Err(ErrorCode::GenericError),
    }
}

fn page_to_address(page_number: usize) -> Option<usize> {
    let offset = page_number.checked_mul(CY_FLASH_ROW_SIZE)?;
    if offset >= CYBOOT_FLASH_TOTAL_SIZE {
        return None;
    }
    CYBOOT_FLASH_BASE_ADDR.checked_add(offset)
}

pub struct FlashPage(pub [u8; CY_FLASH_ROW_SIZE]);

impl Default for FlashPage {
    fn default() -> Self {
        Self([0; CY_FLASH_ROW_SIZE])
    }
}

impl AsMut<[u8]> for FlashPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub struct Flash {
    initialized: Cell<bool>,
    refresh_enabled: Cell<bool>,
    context: Cell<Option<CybootFlashContext>>,
    refresh: Cell<Option<CybootFlashRefresh>>,
}

impl Flash {
    pub const fn new() -> Self {
        Self {
            initialized: Cell::new(false),
            refresh_enabled: Cell::new(false),
            context: Cell::new(Some(CybootFlashContext::blocking_default())),
            refresh: Cell::new(Some(CybootFlashRefresh {
                min_count: 0,
                max_count: 0,
                min_page_addr: 0,
                scratch_row_idx: 0,
            })),
        }
    }

    pub fn init(&self, refresh_enable: bool) -> Result<(), ErrorCode> {
        if self.initialized.get() {
            if refresh_enable && !self.refresh_enabled.get() {
                self.enable_refresh()?;
            }
            return Ok(());
        }

        let Some(mut ctx) = self.context.take() else {
            return Err(ErrorCode::BadState);
        };
        ctx.callback_pre_irq = flash_noop_callback;
        ctx.callback_post_irq = flash_noop_callback;
        ctx.callback_complete = flash_noop_callback;
        ctx.flags = CybootFlashMode::Blocking as u32;
        self.context.set(Some(ctx));

        self.initialized.set(true);
        if refresh_enable {
            self.enable_refresh()?;
        }

        Ok(())
    }

    fn enable_refresh(&self) -> Result<(), ErrorCode> {
        let Some(mut ctx) = self.context.take() else {
            return Err(ErrorCode::BadState);
        };
        let Some(mut refresh) = self.refresh.take() else {
            self.context.set(Some(ctx));
            return Err(ErrorCode::BadState);
        };

        ctx.refresh = ptr::from_mut(&mut refresh);

        // ### Safety
        // ROM table entries are trusted fixed function pointers; the pointers
        // passed here are valid for the duration of these calls.
        unsafe {
            (ROM_FUNC.cyboot_flash_refresh_init)(SECTOR_INDEX_WITHOUT_SFLASH, &mut refresh);
        }

        // ### Safety
        // Same argument validity as above; recover operates on the same sector
        // and refresh state.
        let recover_status = unsafe {
            (ROM_FUNC.cyboot_flash_refresh_recover)(SECTOR_INDEX_WITHOUT_SFLASH, &mut refresh)
        };

        self.refresh.set(Some(refresh));
        self.context.set(Some(ctx));
        self.refresh_enabled.set(true);
        map_rom_status(recover_status)
    }
}

impl FlashSpeHil for Flash {
    type Page = FlashPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        let Some(address) = page_to_address(page_number) else {
            return Err((ErrorCode::InvalidArgument, buf));
        };

        // ### Safety
        // `address` is validated to lie inside the PSC3 flash main range and
        // `buf` points to a valid writable page-sized destination.
        unsafe {
            ptr::copy_nonoverlapping(address as *const u8, buf.0.as_mut_ptr(), CY_FLASH_ROW_SIZE);
        }

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if let Err(e) = self.init(self.refresh_enabled.get()) {
            return Err((e, buf));
        }

        let Some(address) = page_to_address(page_number) else {
            return Err((ErrorCode::InvalidArgument, buf));
        };

        let mut row_words = [0u32; FLASH_ROW_AND_COL33_WORDS];
        // ### Safety
        // `row_words` is a contiguous local allocation; viewing it as bytes for
        // exactly its size is valid and does not outlive `row_words`.
        let row_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                row_words.as_mut_ptr().cast::<u8>(),
                CY_FLASH_ROW_SIZE + CYBOOT_FLASH_COLUMN33_SIZE,
            )
        };
        row_bytes[..CY_FLASH_ROW_SIZE].copy_from_slice(&buf.0);

        let Ok(address_u32) = u32::try_from(address) else {
            return Err((ErrorCode::InvalidArgument, buf));
        };

        let Some(mut ctx) = self.context.take() else {
            return Err((ErrorCode::BadState, buf));
        };
        ctx.flags = CybootFlashMode::Blocking as u32;

        // ### Safety
        // The ROM function pointer originates from a fixed ROM table and expects
        // a valid flash row address, a pointer to row data, and a mutable
        // context pointer, all of which are provided here.
        let status =
            unsafe { (ROM_FUNC.cyboot_flash_write_row)(address_u32, row_words.as_ptr(), &mut ctx) };
        self.context.set(Some(ctx));
        map_rom_status(status).map_err(|e| (e, buf))
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        self.init(self.refresh_enabled.get())?;

        let Some(address) = page_to_address(page_number) else {
            return Err(ErrorCode::InvalidArgument);
        };

        let address_u32 = u32::try_from(address).map_err(|_| ErrorCode::InvalidArgument)?;

        let Some(mut ctx) = self.context.take() else {
            return Err(ErrorCode::BadState);
        };
        ctx.flags = CybootFlashMode::Blocking as u32;

        // ### Safety
        // The ROM function pointer is read from the fixed ROM table and called
        // with a validated row-aligned address and a valid context pointer.
        let status = unsafe { (ROM_FUNC.cyboot_flash_erase_row)(address_u32, &mut ctx) };
        self.context.set(Some(ctx));
        map_rom_status(status)
    }
}
