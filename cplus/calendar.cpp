#include <stdio.h>

static const char* months[] = {"January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"};
static const int days[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
static const char* week[] = {"Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"};

int main()
{
    int year = 2026;
    printf("<html>\n");
    printf("<head><title>%d Journal</title></head>\n", year);
    printf("<body>\n");
    printf("<h1>%d Journal</h1>\n\n", year);
    int w = 4; // day of the week of Jan 1, sunday=0
    for (int m = 0; m < 12; m++)
    {
        int daysInMonth = days[m];
        if (m == 1 && (year % 4) == 0)
            daysInMonth = 29;
        printf("<h2>%s</h2>\n\n", months[m]);
        for (int d = 0; d < daysInMonth; d++)
        {
            printf("<h3>%s, %s %d, %d</h3>\n\n", week[w % 7], months[m], d+1, year);
            w++;
        }
    }
    printf("</body>\n");
    printf("</html>\n");
}
