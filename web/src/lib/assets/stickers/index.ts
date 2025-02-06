import xdsecMascotCrying from "./xdsec-mascot-crying.webp";
import xdsecMascotDead from "./xdsec-mascot-dead.gif";
import xdsecMascotHappy from "./xdsec-mascot-happy.webp";
import xdsecMascotLoading from "./xdsec-mascot-loading.gif";
import xdsecMascotNormal from "./xdsec-mascot-normal.webp";
import xdsecMascotSparkle from "./xdsec-mascot-sparkle.gif";
import xdsecMascotSpining from "./xdsec-mascot-spining.gif";
import xdsecMascotUnsee from "./xdsec-mascot-unsee.webp";

export type Sticker = {
  src: string;
  alt: string;
};

export const stickerSet: Sticker[] = [
  { src: xdsecMascotCrying, alt: "Crying" },
  { src: xdsecMascotHappy, alt: "Happy" },
  { src: xdsecMascotNormal, alt: "Stare" },
  { src: xdsecMascotUnsee, alt: "Wink" },
  { src: xdsecMascotDead, alt: "Dead" },
  { src: xdsecMascotLoading, alt: "Loading" },
  { src: xdsecMascotSparkle, alt: "Sparkle" },
  { src: xdsecMascotSpining, alt: "Spining" },
];
