from ast import Raise

import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

plt.style.use("dark_background")

df_type1 = pd.read_csv("TYPEI_DATA.csv", header=0)
df_type2 = pd.read_csv("TYPEII_DATA.csv", header=0)

# x_type1,y_type1 => dtype = datetime64[ns] (numpy)
x_type1 = pd.to_datetime(df_type1["Departure Date [JD]"], origin="julian", unit="D")
y_type1 = pd.to_datetime(df_type1["Arrival Date [JD]"], origin="julian", unit="D")

x_type2 = pd.to_datetime(df_type2["Departure Date [JD]"], origin="julian", unit="D")
y_type2 = pd.to_datetime(df_type2["Arrival Date [JD]"], origin="julian", unit="D")

# Convert BEFORE passing to tricontour
x_type1_num = mdates.date2num(x_type1.dt.to_pydatetime())
y_type1_num = mdates.date2num(y_type1.dt.to_pydatetime())
x_type2_num = mdates.date2num(x_type2.dt.to_pydatetime())
y_type2_num = mdates.date2num(y_type2.dt.to_pydatetime())

# TOF
tof_type1 = y_type1_num - x_type1_num

# Type I and II Departure and Arrival Energies (C3) [km^2/s^2]
C3_dep_type1 = df_type1["Departure C3 [km^2/s^2]"]
C3_arr_type1 = df_type1["Arrival C3 [km^2/s^2]"]
C3_dep_type2 = df_type2["Departure C3 [km^2/s^2]"]
C3_arr_type2 = df_type2["Arrival C3 [km^2/s^2]"]

# Type I and II Departure and Arrival Delta-V [km/s]
deltaV_dep_type1 = np.sqrt(C3_dep_type1)


def plot_tof_vs_DV():
    plt.figure(2)
    if tof_type1 is not None and deltaV_dep_type1 is not None:
        pmin = np.percentile(deltaV_dep_type1, 5)
        pmax = np.percentile(deltaV_dep_type1, 40)
        clipped = np.clip(deltaV_dep_type1, a_min=pmin, a_max=pmax)
        plt.scatter(tof_type1, clipped, s=0.1)
    plt.title("TOF vs. Delta-V")
    plt.ylabel("Delta-V [km/s]")
    plt.xlabel("Time of Flight (TOF) [Days]")
    plt.grid(True, linestyle="dotted", alpha=0.3)
    plt.savefig("tof_vs_deltaV.png", dpi=300)


def plot(lw, levels, dep_max_C3, arr_max_C3, plot_arrival=False):

    # Departure C3 Type 1
    C3_dep_type1_cap = np.clip(C3_dep_type1, a_min=None, a_max=dep_max_C3)
    # Departure C3 Type 2
    C3_dep_type2_cap = np.clip(C3_dep_type2, a_min=None, a_max=dep_max_C3)

    # Arrival C3 Type 1
    C3_arr_type1_cap = np.clip(C3_arr_type1, a_min=None, a_max=arr_max_C3)
    # Arrival C3 Type 2
    C3_arr_type2_cap = np.clip(C3_arr_type2, a_min=None, a_max=arr_max_C3)

    fig, ax = plt.subplots(figsize=(10, 8))

    # Type 1 Departure Energy Contour Lines
    type1_dep_lines = ax.tricontour(
        x_type1_num,  # <-- original floats from Julian Date
        y_type1_num,
        C3_dep_type1_cap,
        levels=levels,
        colors="white",
        linewidths=lw,
    )
    ax.clabel(type1_dep_lines, inline=True, fontsize=10, fmt="%.0f", colors="white")

    # Type 2 Departure Energy Contour Lines
    type2_dep_lines = ax.tricontour(
        x_type2_num,
        y_type2_num,
        C3_dep_type2_cap,
        levels=levels,
        colors="white",
        linewidths=lw,
    )
    ax.clabel(type2_dep_lines, inline=True, fontsize=10, fmt="%.0f", colors="white")

    if plot_arrival:
        # Type 1 Arrival Energy Contour Lines
        type1_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            C3_arr_type1_cap,
            levels=levels,
            colors="red",
            linewidths=lw,
            alpha=0.5,
        )
        ax.clabel(type1_arr_lines, inline=True, fontsize=10, fmt="%.0f", colors="red")

        # Type 2 Arrival Energy Contour Lines
        type2_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            C3_arr_type2_cap,
            levels=levels,
            colors="red",
            linewidths=lw,
            alpha=0.5,
        )
        ax.clabel(type2_arr_lines, inline=True, fontsize=10, fmt="%.0f", colors="red")

    """
    # TOF Contours Lines
    ax.set_xlim(x_type1_num.min(), x_type1_num.max())  # set limits
    ax.set_ylim(y_type1_num.min(), y_type1_num.max())
    x_right = ax.get_xlim()[1]  # right edge x value
    y_bottom, y_top = ax.get_ylim()
    for tof in np.arange(100, 600, 100):
        y_line = x_type1_num + tof  # y = x + TOF (constant TOF diagonal)
        ax.plot(x_type1_num, y_line, "g", lw=0.3)

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
                fontsize=10,
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
                    fontsize=10,
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
    ax.tick_params(axis="x", rotation=45, labelsize=10)
    ax.tick_params(axis="y", labelsize=10)

    # Plot Formatting
    plt.tight_layout()
    plt.title("Ballistic Transfer Trajectories")
    plt.xlabel("Departure Date", weight="bold")
    plt.ylabel("Arrival Date", weight="bold")
    fig.autofmt_xdate(rotation=45, ha="right")
    plt.grid(True, linestyle="dotted", alpha=0.3)
    plt.savefig("porkchop_plot.png", dpi=300)


if __name__ == "__main__":
    plot(lw=0.5, levels=10, dep_max_C3=20, arr_max_C3=20, plot_arrival=True)
    plot_tof_vs_DV()
