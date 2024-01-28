import cv2
import matplotlib.pyplot as plt
import pandas as pd


def load_image(file_name):
    cv_img = cv2.imread(file_name)
    #     return cv2.resize(cv_img, dsize=(224, 224), interpolation=cv2.INTER_CUBIC)
    return cv2.cvtColor(cv_img, cv2.COLOR_BGR2RGB)


def load_label(file_name):
    df = pd.read_csv(file_name + ".cat", sep=" ", header=None)
    df = df.drop(columns=[0, 19])
    return df.values.squeeze()


if __name__ == '__main__':
    file = 'datasets/CAT_00/00000001_000.jpg'
    image_in_numpy = load_image(file)
    label_in_numpy = load_label(file)

    plt.scatter(label_in_numpy[::2], label_in_numpy[1::2], color='yellow')
    plt.imshow(image_in_numpy)
    plt.show()
    print(label_in_numpy)
