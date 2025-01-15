import { themeStore } from "@storage/theme";
import { type ComponentProps, onMount } from "solid-js";

export default function (props: ComponentProps<"svg">) {
  let poly1: SVGPolygonElement;
  let poly2: SVGPolygonElement;
  let poly3: SVGPolygonElement;
  const { width, height } = { width: 256, height: 256, ...props };
  onMount(() => {
    import("./styles/vn-animate.scss");
    poly1!.dataset.fade = "a";
    poly2!.dataset.fade = "b";
    poly3!.dataset.fade = "c";
  });
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      data-vn-animate-logo
      viewBox="0 0 281.26 195.4"
      width={width}
      height={height}
      {...props}
    >
      <g class="vn-animate-box" fill={themeStore.colorScheme === "dark" ? "#fff" : "#0a1d38"}>
        <g>
          <polygon data-fade="a-" points="0 0 96.09 192.92 96.09 132.46 0 0" ref={poly1!}></polygon>
          <polygon data-fade="b-" points="102.64 132.56 102.64 192.92 199.32 0 102.64 132.56" ref={poly2!}></polygon>
          <polygon data-fade="c-" points="188.76 35.12 206.06 0 281.26 195.4 188.76 35.12" ref={poly3!}></polygon>
        </g>
      </g>
    </svg>
  );
}
