import plotly.graph_objects as go
import websockets
from nicegui import ui, app

import gzip
import struct
import base64

imgb64 = ""


@ui.refreshable
def generate_plot():
    with ui.element("div").style("width: auto; height: 100%; "):
        fig = go.Figure(go.Scatter(x=[1, 2, 3, 4], y=[1, 2, 3, 2.5]))

        fig.update_layout(
            dragmode="pan",
            margin=dict(l=0, r=0, t=0, b=0),
            showlegend=False,
            xaxis=dict(visible=False),
            yaxis=dict(visible=False),
        )

        if imgb64 != "":
            fig.add_layout_image(
                dict(
                    source="data:image/png;base64,{}".format(imgb64.decode()),
                    xref="x",
                    yref="y",
                    x=0,  # Adjust x position as needed
                    y=0,  # Adjust y position as needed
                    # sizex=1,  # Adjust width as needed
                    # sizey=1,  # Adjust height as needed
                    opacity=1,
                    layer="below",
                )
            )

        plot = ui.plotly(fig).classes("aspect-square object-contain")
        plot.on("plotly_click", ui.notify)

        config = {"scrollZoom": True, "displayModeBar": False}
        fig = fig.to_dict()
        fig["config"] = config

        return fig


async def ws_client():
    url = "ws://127.0.0.1:7890"

    async with websockets.connect(url) as ws:
        print("WebSocket: Client Connected.")

        # await ws.send(f"{age}")

        # Stay alive forever, listen to incoming msgs
        while True:
            msg = await ws.recv()

            if msg[:2] == b"\x1f\x8b":

                decompressed_data = gzip.decompress(msg)  # type: ignore
                print(decompressed_data)

                if decompressed_data[:2] == b"MU":
                    global imgb64
                    x = struct.unpack(">q", decompressed_data[2:10])
                    y = struct.unpack(">q", decompressed_data[10:18])
                    img = decompressed_data[18:]

                    imgb64 = base64.b64encode(img)
                    generate_plot.refresh()
            else:
                print(msg)


ui.label("SLAM Robot Control")

with ui.row(wrap=False, align_items="center"):
    with ui.card().style("width: 25vw; height: 90vh"):
        ui.label("Settings")

    with ui.card().style("width: 70vw; height: 90vh; object-position: center;"):
        fig = generate_plot()


app.on_startup(ws_client)
ui.run()
