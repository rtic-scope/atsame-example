MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM : ORIGIN = 0x20010000, LENGTH = 64K
  CAN (rw) : ORIGIN = 0x20000000, LENGTH = 64K
  ECC : ORIGIN = 0x20020000, LENGTH = 128K
}
