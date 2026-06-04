import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

plt.style.use("dark_background")


def dual_plot(lw, levels, dep_max_C3, arr_max_C3, arrival):
    df_type1 = pd.read_csv("TYPEI_DATA.csv", header=0)
    df_type2 = pd.read_csv("TYPEII_DATA.csv", header=0)

    # x_type1,y_type1 => dtype = datetime64[ns] (numpy)
    x_type1 = pd.to_datetime(df_type1["Departure Date [JD]"], origin="julian", unit="D")
    y_type1 = pd.to_datetime(df_type1["Arrival Date [JD]"], origin="julian", unit="D")
    z_type1 = df_type1["Departure C3 [km^2/s^2]"]
    z2_type1 = df_type1["Arrival C3 [km^2/s^2]"]

    x_type2 = pd.to_datetime(df_type2["Departure Date [JD]"], origin="julian", unit="D")
    y_type2 = pd.to_datetime(df_type2["Arrival Date [JD]"], origin="julian", unit="D")
    z_type2 = df_type2["Departure C3 [km^2/s^2]"]
    z2_type2 = df_type2["Arrival C3 [km^2/s^2]"]

    # Departure C3 Short
    z_type1_capped = np.clip(z_type1, a_min=None, a_max=dep_max_C3)
    # Departure C3 Long
    z_type2_capped = np.clip(z_type2, a_min=None, a_max=dep_max_C3)

    # Arrival C3 Short
    z2_type1_capped = np.clip(z2_type1, a_min=None, a_max=arr_max_C3)
    # Arrival C3 Long
    z2_type2_capped = np.clip(z2_type2, a_min=None, a_max=arr_max_C3)

    # Convert BEFORE passing to tricontour
    x_type1_num = mdates.date2num(x_type1.dt.to_pydatetime())
    y_type1_num = mdates.date2num(y_type1.dt.to_pydatetime())
    x_type2_num = mdates.date2num(x_type2.dt.to_pydatetime())
    y_type2_num = mdates.date2num(y_type2.dt.to_pydatetime())

    fig, ax = plt.subplots(figsize=(6, 6))

    type1_dep_lines = ax.tricontour(
        x_type1_num,  # <-- original floats
        y_type1_num,
        z_type1_capped,
        levels=levels,
        colors="white",
        linewidths=lw,
    )
    ax.clabel(type1_dep_lines, inline=True, fontsize=7, fmt="%.0f", colors="white")

    type2_dep_lines = ax.tricontour(
        x_type2_num,
        y_type2_num,
        z_type2_capped,
        levels=levels,
        colors="white",
        linewidths=lw,
    )
    ax.clabel(type2_dep_lines, inline=True, fontsize=7, fmt="%.0f", colors="white")

    if arrival:
        type1_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            z2_type1_capped,
            levels=levels,
            colors="red",
            linewidths=lw,
            alpha=0.4,
        )
        ax.clabel(type1_arr_lines, inline=True, fontsize=7, fmt="%.0f", colors="red")

        type2_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            z2_type2_capped,
            levels=levels,
            colors="red",
            linewidths=lw,
            alpha=0.4,
        )
        ax.clabel(type2_arr_lines, inline=True, fontsize=7, fmt="%.0f", colors="red")
        
    """
    # TOF Contours Lines
    ax.set_xlim(x_type1_num.min(), x_type1_num.max())  # set limits
    ax.set_ylim(y_type1_num.min(), y_type1_num.max())


    x_right = ax.get_xlim()[1]  # right edge x value
    y_bottom, y_top = ax.get_ylim()
    for tof in np.arange(100, 600, 100):
        y_line = x_type1_num + tof  # y = x + TOF (constant TOF diagonal)
        ax.plot(x_type1_num, y_line, "g", lw=0.7)

        # Find where the line intersects the right edge or top edge
        y_at_right = x_right + tof  # y value when x = x_right

        if y_bottom <= y_at_right <= y_top:
            # Line exits through the right edge
            ax.annotate(
                str(int(tof)),
                xy=(x_right, y_at_right),
                xytext=(5, 0),  # offset in points to the right
                textcoords="offset points",
                va="center",
                ha="left",
                fontsize=8,
                color="g",
                annotation_clip=False,  # allow label outside axes
            )
        else:
            # Line exits through the top edge — find x at y_top
            x_at_top = y_top - tof
            if ax.get_xlim()[0] <= x_at_top <= x_right:
                ax.annotate(
                    str(int(tof)),
                    xy=(x_at_top, y_top),
                    xytext=(0, 5),
                    textcoords="offset points",
                    va="bottom",
                    ha="center",
                    fontsize=8,
                    color="g",
                    annotation_clip=False,
                )

    """
    # Axis Date Formatting
    ax.xaxis.set_major_locator(mdates.MonthLocator(interval=1))
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%d-%b-%Y"))

    ax.yaxis.set_major_locator(mdates.MonthLocator(interval=3))
    ax.yaxis.set_major_formatter(mdates.DateFormatter("%d-%b-%Y"))

    # Axis Label Formatting
    ax.tick_params(axis="x", rotation=45, labelsize=8)
    ax.tick_params(axis="y", labelsize=8)

    # Plot Formatting
    plt.tight_layout()
    plt.title("Ballistic Transfer Trajectories")
    plt.xlabel("Departure Date", weight="bold")
    plt.ylabel("Arrival Date", weight="bold")
    fig.autofmt_xdate(rotation=45, ha="right")
    plt.grid(True, linestyle="dotted", alpha=0.5)
    plt.show()


if __name__ == "__main__":
    levels = [10, 12, 14, 16, 18, 20, 25, 30, 35, 40, 45, 50]
    dual_plot(lw=0.5, levels=levels, dep_max_C3=50, arr_max_C3=50, arrival=False)
