

SECTIONS {

  .text : ALIGN(4)
  {
    . = ALIGN(4);
    *(.literal .text .literal.* .text.*)
    . = ALIGN(4);
  } > ROTEXT

}