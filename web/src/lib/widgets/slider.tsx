import { Slider, type SliderRootProps } from "@ark-ui/solid";
import { splitProps } from "solid-js";

export type SliderProps = {
    label?: string;
};

export default function (props: SliderRootProps & SliderProps) {
    const [sliderProps, others] = splitProps(props, ["label"]);
    return (
        <Slider.Root
            {...others}
            class={`slider ${props.class ?? ""} ${others.orientation === "vertical" ? "slider-vertical" : ""}`.trim()}
        >
            <div class="label slider-label">
                <Slider.Label>{sliderProps.label}</Slider.Label>
                <Slider.ValueText />
            </div>
            <Slider.Control class="slider-control group">
                <Slider.Track class="slider-track">
                    <Slider.Range class="slider-range" />
                </Slider.Track>
                <Slider.Thumb index={0} class="slider-thumb group-hover:border-2">
                    <Slider.HiddenInput />
                </Slider.Thumb>
            </Slider.Control>
        </Slider.Root>
    );
}
