export function inView (node: HTMLElement, params: { root?: HTMLElement, top?: number, bottom?: number } = {}) {
    let observer: IntersectionObserver

    const handleIntersect: IntersectionObserverCallback = (e) => {
        const v = e[0].isIntersecting ? "enter" : "exit"
        node.dispatchEvent(new CustomEvent(v))
    };

    const setObserver: (object: { root?: HTMLElement, top?: number, bottom?: number }) => void = ({ root, top, bottom }) => {
        const marginTop = top ? top * -1 : 0
        const marginBottom = bottom ? bottom * -1 : 0
        const rootMargin = `${marginTop}px 0px ${marginBottom}px 0px`
        const options = { root, rootMargin }
        if (observer) observer.disconnect()
        observer = new IntersectionObserver(handleIntersect, options)
        observer.observe(node)
    }

    setObserver(params)

    return {
        update (params: { root?: HTMLElement, top?: number, bottom?: number }) {
            setObserver(params)
        },

        destroy () {
            if (observer) observer.disconnect()
        }
    };
}