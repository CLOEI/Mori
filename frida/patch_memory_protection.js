const module = Process.getModuleByName("ntdll.dll")
const ZwProtectVirtualMemoryAddress = module.findExportByName("ZwProtectVirtualMemory");
console.log("ZwProtectVirtualMemory address: " + ZwProtectVirtualMemoryAddress);

const patchBytes = [0x4C, 0x8B, 0xD1, 0xB8, 0x50];
const patchSize = patchBytes.length;

Memory.protect(ZwProtectVirtualMemoryAddress, patchSize, 'rwx');
ZwProtectVirtualMemoryAddress.writeByteArray(patchBytes);