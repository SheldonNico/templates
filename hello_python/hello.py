# import click, time
#
# @click.command()
# @click.option("--count", default=1, help="Number of greetings.")
# @click.option("--name", prompt="Your name", help="The person to greet")
# def hello(count, name):
#     with click.progressbar([1, 2, 3]) as bar:
#         for _ in bar:
#             time.sleep(1)
#             # print(name)
#
# if __name__ == "__main__":
#     hello()

if __name__ == "__main__":
    import pygal
    bar_chart = pygal.Bar()
    bar_chart.add('Fibonacci', [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55])
    bar_chart.render_to_png('bar_chart.png')
    # bar_chart.render_to_file('bar_chart.svg')
