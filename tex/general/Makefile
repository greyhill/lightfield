.PHONY: clean

general.pdf: general.tex light,field,geometry.tex tab,xport.tex
	latexmk -pdf general

clean:
	latexmk -C -pdf general
